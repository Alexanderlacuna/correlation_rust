// this file contains code to read from ldmb file:

use lmdb::{Database, Environment, Transaction};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct LmdbMetadata {
    pub rows: usize,
    pub columns: usize,
    pub dtype: String,
    pub traits: Vec<String>,
    pub strains: Vec<String>,
    pub has_se: bool,
    pub dataset_name: String,
}

pub struct LmdbReader {
    env: Environment,
}

impl LmdbReader {
    pub fn new(lmdb_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let env = Environment::new()
            .set_max_dbs(10)
            .open(Path::new(lmdb_path))?;

        Ok(Self { env })
    }

    pub fn read_metadata(&self) -> Result<LmdbMetadata, Box<dyn std::error::Error>> {
        let db = self.env.open_db(None)?;
        let txn = self.env.begin_ro_txn()?;

        let metadata_bytes = txn
            .get(db, &b"probeset_metadata")
            .map_err(|e| format!("Metadata not found: {}", e))?;

        let metadata: LmdbMetadata = serde_json::from_slice(metadata_bytes)?;

        Ok(metadata)
    }

    pub fn read_matrix(&self) -> Result<(Vec<Vec<f64>>, Vec<String>), Box<dyn std::error::Error>> {
        let db = self.env.open_db(None)?;
        let txn = self.env.begin_ro_txn()?;

        // Read metadata
        let metadata_bytes = txn.get(db, &b"probeset_metadata")?;
        let metadata: LmdbMetadata = serde_json::from_slice(metadata_bytes)?;

        // Read matrix bytes
        let matrix_bytes = txn.get(db, &b"probeset_matrix")?;

        // Convert bytes to f64 array
        let float_count = metadata.rows * metadata.columns;
        let mut floats = Vec::with_capacity(float_count);

        for chunk in matrix_bytes.chunks_exact(8) {
            let bytes: [u8; 8] = chunk.try_into()?;
            floats.push(f64::from_le_bytes(bytes));
        }

        // Reshape to 2D matrix (row-major order)
        let mut matrix = Vec::with_capacity(metadata.rows);
        for i in 0..metadata.rows {
            let start = i * metadata.columns;
            let end = start + metadata.columns;
            matrix.push(floats[start..end].to_vec());
        }

        Ok((matrix, metadata.traits))
    }

    /// Read a single trait row by name
    pub fn read_trait(&self, trait_name: &str) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        let db = self.env.open_db(None)?;
        let txn = self.env.begin_ro_txn()?;

        // Read metadata
        let metadata_bytes = txn.get(db, &b"probeset_metadata")?;
        let metadata: LmdbMetadata = serde_json::from_slice(metadata_bytes)?;

        // Find trait index
        let trait_idx = metadata
            .traits
            .iter()
            .position(|t| t == trait_name)
            .ok_or_else(|| format!("Trait '{}' not found", trait_name))?;

        // Read matrix bytes
        let matrix_bytes = txn.get(db, &b"probeset_matrix")?;

        // Calculate byte offset for this row
        let bytes_per_value = 8; // f64 is 8 bytes
        let bytes_per_row = metadata.columns * bytes_per_value;
        let offset = trait_idx * bytes_per_row;

        // Extract the row
        let row_bytes = &matrix_bytes[offset..offset + bytes_per_row];
        let mut row = Vec::with_capacity(metadata.columns);

        for chunk in row_bytes.chunks_exact(8) {
            let bytes: [u8; 8] = chunk.try_into()?;
            row.push(f64::from_le_bytes(bytes));
        }

        Ok(row)
    }

    /// Iterator-style reader for streaming through all traits
    pub fn iter_traits(&self) -> Result<LmdbTraitIterator, Box<dyn std::error::Error>> {
        let metadata = self.read_metadata()?;
        let db = self.env.open_db(None)?;
        let txn = self.env.begin_ro_txn()?;

        let matrix_bytes = txn.get(db, &b"probeset_matrix")?.to_vec();

        Ok(LmdbTraitIterator {
            matrix_bytes,
            metadata,
            current_row: 0,
        })
    }
}

pub struct LmdbTraitIterator {
    matrix_bytes: Vec<u8>,
    metadata: LmdbMetadata,
    current_row: usize,
}

impl Iterator for LmdbTraitIterator {
    type Item = (String, Vec<f64>); // (trait_name, values)

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.metadata.rows {
            return None;
        }

        let bytes_per_value = 8;
        let bytes_per_row = self.metadata.columns * bytes_per_value;
        let offset = self.current_row * bytes_per_row;

        let row_bytes = &self.matrix_bytes[offset..offset + bytes_per_row];
        let mut row = Vec::with_capacity(self.metadata.columns);

        for chunk in row_bytes.chunks_exact(8) {
            let bytes: [u8; 8] = chunk.try_into().ok()?;
            row.push(f64::from_le_bytes(bytes));
        }

        let trait_name = self.metadata.traits[self.current_row].clone();
        self.current_row += 1;

        Some((trait_name, row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Global LMDB path for all tests
    const LMDB_PATH: &str = "/home/kabui/lmdb_data/GTEx_Liver_0414/GTEx_Liver_0414/";

    /// Helper to check if LMDB test data exists
    fn lmdb_exists() -> bool {
        std::path::Path::new(LMDB_PATH).exists()
    }

    #[test]
    fn test_lmdb_reader() {
        if !lmdb_exists() {
            eprintln!("Skipping test: LMDB not found at {}", LMDB_PATH);
            return;
        }
        let reader = LmdbReader::new(LMDB_PATH);
        assert!(reader.is_ok());
    }

    #[test]
    fn test_read_metadata() {
        if !lmdb_exists() {
            eprintln!("Skipping test: LMDB not found at {}", LMDB_PATH);
            return;
        }
        let reader = LmdbReader::new(LMDB_PATH).unwrap();
        let metadata = reader.read_metadata();
        assert!(metadata.is_ok());

        if let Ok(meta) = metadata {
            println!("\n=== LMDB Metadata ===");
            println!("Dataset: {}", meta.dataset_name);
            println!("Shape: {} traits × {} strains", meta.rows, meta.columns);
            println!("Has SE: {}", meta.has_se);
            println!("Dtype: {}", meta.dtype);
            println!("\nFirst 5 traits:");
            for (i, trait_name) in meta.traits.iter().take(5).enumerate() {
                println!("  {}: {}", i + 1, trait_name);
            }
            println!("\nFirst 10 strains:");
            for (i, strain) in meta.strains.iter().take(10).enumerate() {
                println!("  {}: {}", i + 1, strain);
            }
            println!("===================\n");
        }
    }

    #[test]
    fn test_read_matrix_shape() {
        if !lmdb_exists() {
            eprintln!("Skipping test: LMDB not found at {}", LMDB_PATH);
            return;
        }
        let reader = LmdbReader::new(LMDB_PATH).unwrap();
        let result = reader.read_matrix();
        assert!(result.is_ok());

        if let Ok((matrix, trait_names)) = result {
            println!("\n=== Matrix Information ===");
            println!("Matrix rows (traits): {}", matrix.len());
            println!("Matrix cols (strains): {}", matrix[0].len());
            println!("Total elements: {}", matrix.len() * matrix[0].len());

            println!("\n=== First 5 Traits (Top 10 values each) ===");
            for (i, row) in matrix.iter().take(5).enumerate() {
                println!("\nTrait {}: {}", i + 1, trait_names[i]);
                print!("  Values: ");
                for (j, val) in row.iter().take(10).enumerate() {
                    if j > 0 {
                        print!(", ");
                    }
                    print!("{:.2}", val);
                }
                if row.len() > 10 {
                    print!(" ... ({} more)", row.len() - 10);
                }
                println!();
            }

            println!("\n=== Statistics for First Row ===");
            if !matrix.is_empty() && !matrix[0].is_empty() {
                let first_row = &matrix[0];
                let min = first_row.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = first_row.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let sum: f64 = first_row.iter().sum();
                let mean = sum / first_row.len() as f64;

                println!("  Min: {:.2}", min);
                println!("  Max: {:.2}", max);
                println!("  Mean: {:.2}", mean);
            }
            println!("==========================\n");
        }
    }

    #[test]
    fn test_read_specific_trait() {
        if !lmdb_exists() {
            eprintln!("Skipping test: LMDB not found at {}", LMDB_PATH);
            return;
        }
        let reader = LmdbReader::new(LMDB_PATH).unwrap();
        let metadata = reader.read_metadata().unwrap();

        if let Some(first_trait) = metadata.traits.first() {
            println!("\n=== Reading Specific Trait ===");
            println!("Trait name: {}", first_trait);

            let result = reader.read_trait(first_trait);
            println!("the results are {:?}", result);
            assert!(result.is_ok());

            if let Ok(values) = result {
                println!("Number of values: {}", values.len());
                println!("\nFirst 10 values:");
                for (i, val) in values.iter().take(10).enumerate() {
                    println!("  Strain {}: {:.2}", i + 1, val);
                }

                let nan_count = values.iter().filter(|v| v.is_nan()).count();
                println!(
                    "\nNaN values: {} ({:.1}%)",
                    nan_count,
                    (nan_count as f64 / values.len() as f64) * 100.0
                );
            }
            println!("==============================\n");
        }
    }

    #[test]
    fn test_iterator() {
        if !lmdb_exists() {
            eprintln!("Skipping test: LMDB not found at {}", LMDB_PATH);
            return;
        }
        let reader = LmdbReader::new(LMDB_PATH).unwrap();
        let mut iter = reader.iter_traits().unwrap();

        println!("\n=== Testing Iterator ===");
        println!("First 3 traits from iterator:\n");

        for i in 0..3 {
            if let Some((trait_name, values)) = iter.next() {
                println!("{}. Trait: {}", i + 1, trait_name);
                println!("   Values: {}", values.len());
                print!("   First 5: ");
                for (j, val) in values.iter().take(5).enumerate() {
                    if j > 0 {
                        print!(", ");
                    }
                    print!("{:.2}", val);
                }
                println!(" ...\n");
            }
        }
        println!("========================\n");
    }
}
