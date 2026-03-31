#!/usr/bin/env python3
"""Check available strains in an LMDB dataset.

Usage:
    python3 check_strains.py /path/to/lmdb/

Example:
    python3 check_strains.py /home/kabui/lmdb_data/HC_M2_0606_P/
"""

import sys
import json

try:
    import lmdb
except ImportError:
    print("Error: lmdb module not found. Install with: pip install lmdb")
    sys.exit(1)


def check_strains(lmdb_path: str):
    """Print strain information from LMDB metadata."""
    try:
        env = lmdb.open(lmdb_path, readonly=True)
        with env.begin() as txn:
            raw = txn.get(b'probeset_metadata')
            if raw is None:
                print(f"Error: probeset_metadata not found in {lmdb_path}")
                sys.exit(1)
            
            meta = json.loads(raw)
            
            print(f"Dataset: {meta.get('dataset_name', 'N/A')}")
            print(f"Shape: {meta['rows']} traits × {meta['columns']} strains")
            print()
            
            strains = meta.get('strains', [])
            print(f"Available strains ({len(strains)} total):")
            
            # Check if using IDs or names
            sample = strains[0] if strains else ""
            is_numeric = sample.isdigit() if isinstance(sample, str) else False
            
            if is_numeric:
                print("NOTE: This LMDB uses numeric strain IDs (old format).")
                print("      Re-dump with updated batch_lmdb_metadata.py for names.\n")
            else:
                print("NOTE: This LMDB uses strain names (new format).\n")
            
            # Print first 20 strains
            for i, s in enumerate(strains[:20]):
                print(f'  [{i:2d}] "{s}"')
            
            if len(strains) > 20:
                print(f"  ... and {len(strains) - 20} more")
            
            # If strain_id_to_name mapping exists, show it
            if 'strain_id_to_name' in meta:
                print("\nID to Name mapping (first 10):")
                for i, (k, v) in enumerate(list(meta['strain_id_to_name'].items())[:10]):
                    print(f'  {k} -> "{v}"')
    
    except Exception as e:
        print(f"Error reading LMDB: {e}")
        sys.exit(1)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        print(f"Usage: {sys.argv[0]} /path/to/lmdb/")
        sys.exit(1)
    
    check_strains(sys.argv[1])
