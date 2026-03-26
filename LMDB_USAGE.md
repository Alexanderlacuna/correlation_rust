# LMDB Correlation Usage Guide

This guide explains how to use the LMDB correlation feature in correlation_rust.

## Overview

The LMDB correlation mode allows you to:
- Skip CSV file generation
- Read expression data directly from memory-mapped LMDB files
- Correlate with any subset of strains by specifying them in the JSON input

## JSON Input Format

### With Strains (LMDB Mode)

Use this format when you have an LMDB dataset and want to correlate with specific strains:

```json
{
  "file_path": "/path/to/lmdb/dataset/",
  "x_vals": [1.2, 3.4, 5.6, 7.8],
  "strains": ["BXD1", "BXD2", "BXD5", "BXD8"],
  "sample_values": "input_trait_name",
  "method": "pearson",
  "file_delimiter": ",",
  "output_file": "/tmp/results.txt"
}
```

**Important:**
- `file_path` must point to a **directory** (the LMDB folder)
- `strains` array order must match `x_vals` order (1-to-1 correspondence)
- All strains listed must exist in the LMDB dataset

### Without Strains (Legacy CSV Mode)

For backward compatibility, CSV mode still works:

```json
{
  "file_path": "path/to/data.csv",
  "x_vals": [1.2, 3.4, 5.6, 7.8],
  "sample_values": "input_trait_name",
  "method": "pearson",
  "file_delimiter": ",",
  "output_file": "/tmp/results.txt"
}
```

## How It Works

1. **Auto-detection**: Rust checks if `file_path` is a directory
   - If **directory**: LMDB mode with optional strain extraction
   - If **file**: CSV mode (original behavior)

2. **Strain Mapping**: When strains are provided:
   - Read LMDB metadata to get strain order
   - Build strain name → column index map
   - Extract only specified columns for each trait

3. **Correlation**: Compute Pearson or Spearman correlation for each trait

## Example Usage

### Step 1: Prepare LMDB Dataset

Ensure your LMDB dataset is dumped using `batch_lmdb_metadata.py`:

```bash
python batch_lmdb_metadata.py dump-dataset \
    "mysql://user:pass@localhost/db_webqtl" \
    /var/lmdb_data \
    206
```

This creates `/var/lmdb_data/HC_M2_0606_P/` with LMDB files.

### Step 2: Create JSON Input

Create a file `input.json`:

```json
{
  "file_path": "/var/lmdb_data/HC_M2_0606_P/",
  "x_vals": [5.2, 7.1, 3.8, 9.4],
  "strains": ["BXD1", "BXD2", "BXD5", "BXD8"],
  "sample_values": "my_trait",
  "method": "pearson",
  "file_delimiter": ",",
  "output_file": "/tmp/correlation_results.txt"
}
```

### Step 3: Run Correlation

```bash
./correlation_rust input.json /tmp
```

Or via GN3 (when implemented):

```python
from gn3.computations.rust_correlation import run_correlation

results = run_correlation(
    dataset_or_path="/var/lmdb_data/HC_M2_0606_P/",
    trait_vals=[5.2, 7.1, 3.8, 9.4],
    strains=["BXD1", "BXD2", "BXD5", "BXD8"],
    method="pearson",
    delimiter=",",
    tmpdir="/tmp"
)
```

## Output Format

The output file contains one line per correlated trait:

```
trait_name,correlation_coefficient,p_value,num_overlap
100244_at,0.8543,0.0001,4
100253_at,-0.2314,0.7686,4
100341_at,0.9121,0.00001,4
...
```

Fields:
- `trait_name`: Name of the trait (e.g., ProbeSet ID)
- `correlation_coefficient`: Pearson or Spearman r value
- `p_value`: Two-tailed p-value
- `num_overlap`: Number of samples used in correlation

## Error Handling

### Strain Not Found

If a strain specified in the JSON doesn't exist in the LMDB dataset:

```
Error: Strain mismatch: requested 4 strains, found 3. 
Some strains may not exist in the LMDB dataset.
```

**Solution**: Check strain names match exactly (case-sensitive).

### LMDB Path Not Found

```
Error: No such file or directory (os error 2)
```

**Solution**: Verify `file_path` points to the correct LMDB directory.

### Not Enough Samples

Traits with fewer than 4 valid (non-NaN) samples are skipped.

## Performance

| Mode | Time (45K traits × 50 strains) | Notes |
|------|--------------------------------|-------|
| CSV | ~2-5 seconds | File I/O + parsing |
| LMDB (all strains) | ~100-300ms | Memory-mapped |
| LMDB (subset) | ~100-300ms | Column extraction |

## Testing

Run unit tests (no external dependencies):

```bash
cargo test test_compute_with_strains
cargo test test_compute_without_strains
```

Run integration tests (requires GSL and LMDB):

```bash
# Check if LMDB exists
ls /home/kabui/lmdb_data/GTEx_Liver_0414/

# Run ignored tests
cargo test -- --ignored
```

## Architecture

```
input.json
    │
    ▼
┌─────────────────────┐
│   main.rs           │  Entry point
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   analysis.rs       │  Parse JSON, route to Compute
│   - Check strains   │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   correlations.rs   │  Core correlation logic
│   - new_with_strains│  (if strains provided)
│   - compute_from_lmdb│  (LMDB + column extraction)
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   lmdb_reader.rs    │  Read LMDB files
│   - read_metadata   │  (get strain order)
│   - iter_traits     │  (stream all traits)
└─────────────────────┘
```

## Migration from CSV to LMDB

1. **Dump your datasets** using `batch_lmdb_metadata.py`
2. **Update GN2/GN3** to pass LMDB path and strains
3. **Test** with a subset of queries first
4. **Monitor** performance improvements

No changes needed in the Rust binary itself - it auto-detects based on path type!
