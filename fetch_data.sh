#!/usr/bin/env bash
#
# fetch_data.sh - Downloads and extracts Geolife trajectory data
#
# This script downloads the Geolife Trajectories dataset from Microsoft,
# verifies its integrity using SHA256, and extracts specific trajectory data.
#
# Usage: ./fetch_data.sh [output_dir]
#   output_dir: Directory to extract data to (default: ./geolife)
#
# Exit codes:
#   0 - Success
#   1 - Invalid arguments
#   2 - Download failed
#   3 - Checksum verification failed
#   4 - Extraction failed

set -euo pipefail

# Configuration
readonly DEFAULT_OUTPUT_DIR="geolife"
readonly DOWNLOAD_URL="https://download.microsoft.com/download/f/4/8/f4894aa5-fdbc-481e-9285-d5f8c4c4f039/Geolife%20Trajectories%201.3.zip"
readonly EXPECTED_SHA256="1107c5ac064d0a23c8d021a8736a77e53abc75b227062e6260342c6a8d86bdb6"
readonly TRAJECTORY_PATH="Geolife Trajectories 1.3/Data/153/Trajectory"

# Parse command line arguments
output_dir="${1:-$DEFAULT_OUTPUT_DIR}"

# Validate arguments
if [[ ! -d "$(dirname "$output_dir")" ]]; then
    echo "Error: Parent directory of output directory does not exist" >&2
    exit 1
fi

# Create temporary directory
temp_dir="$(mktemp -d)"
temp_file="$temp_dir/archive.zip"

# Cleanup function
cleanup() {
    if [[ -d "$temp_dir" ]]; then
        rm -rf "$temp_dir"
    fi
}

# Register cleanup function
trap cleanup EXIT

# Download the file
echo "Downloading dataset..."
if ! curl -L -o "$temp_file" "$DOWNLOAD_URL"; then
    echo "Error: Failed to download dataset" >&2
    exit 2
fi

# Verify checksum
echo "Verifying checksum..."
if ! echo "$EXPECTED_SHA256  $temp_file" | sha256sum -c -; then
    echo "Error: Checksum verification failed" >&2
    exit 3
fi

# Create output directory if it doesn't exist
mkdir -p "$output_dir"

# Extract specific files
echo "Extracting trajectory data..."
if ! unzip -q "$temp_file" "$TRAJECTORY_PATH/*" -d "$temp_dir"; then
    echo "Error: Failed to extract files" >&2
    exit 4
fi

# Move files to output directory
mv "$temp_dir/$TRAJECTORY_PATH/"* "$output_dir/"

echo "Successfully downloaded and extracted trajectory data to $output_dir"
exit 0
