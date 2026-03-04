#!/bin/bash

# Extract and convert Buf configurations to proto-sign format
# This script helps maintain compatibility with upstream Buf updates
# Note: Only generates *-protosign.yaml files, original buf.yaml files are not copied

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
EXTRACTED_CONFIGS_DIR="${SCRIPT_DIR}/extracted"
BUF_GIT_URL="https://github.com/bufbuild/buf.git"
BUF_BRANCH="${BUF_BRANCH:-main}"  # Allow override via environment variable

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Clone buf project to temporary directory
clone_buf_project() {
    log_info "Creating temporary directory for buf project..."
    BUF_TEMP_DIR=$(mktemp -d -t buf-extract-XXXXXX)
    
    # Ensure cleanup on exit
    trap "cleanup_temp_dir" EXIT
    
    log_info "Cloning buf project from $BUF_GIT_URL (branch: $BUF_BRANCH)..."
    if ! git clone --depth=1 --branch="$BUF_BRANCH" "$BUF_GIT_URL" "$BUF_TEMP_DIR" >/dev/null 2>&1; then
        log_error "Failed to clone buf project"
        log_error "Please check your internet connection and git installation"
        exit 1
    fi
    
    BUF_PROJECT_ROOT="$BUF_TEMP_DIR"
    log_info "Successfully cloned buf project to: $BUF_PROJECT_ROOT"
}

# Cleanup temporary directory
cleanup_temp_dir() {
    if [[ -n "${BUF_TEMP_DIR:-}" && -d "$BUF_TEMP_DIR" ]]; then
        log_info "Cleaning up temporary directory: $BUF_TEMP_DIR"
        rm -rf "$BUF_TEMP_DIR"
    fi
}

# Create output directories
setup_directories() {
    mkdir -p "$EXTRACTED_CONFIGS_DIR"/{main,testdata}
    log_info "Created extraction directories"
}

# Extract main buf.yaml configuration
extract_main_config() {
    local buf_yaml="$BUF_PROJECT_ROOT/buf.yaml"
    local temp_file=$(mktemp)
    local output_file="$EXTRACTED_CONFIGS_DIR/main/proto-sign-main.yaml"
    
    if [[ -f "$buf_yaml" ]]; then
        # Copy to temp file for conversion only
        cp "$buf_yaml" "$temp_file"
        log_info "Found main buf.yaml, converting to proto-sign format"
        
        # Convert directly to proto-sign format (skip copying original)
        convert_to_protosign_format "$temp_file" "$output_file"
        
        # Cleanup temp file
        rm -f "$temp_file"
    else
        log_warn "Main buf.yaml not found"
    fi
}

# Extract test data configurations
extract_testdata_configs() {
    local testdata_dir="$BUF_PROJECT_ROOT/private/bufpkg/bufcheck/testdata/breaking"
    local output_dir="$EXTRACTED_CONFIGS_DIR/testdata"
    
    log_info "Looking for test data in: $testdata_dir"
    
    if [[ ! -d "$testdata_dir" ]]; then
        log_warn "Test data directory not found: $testdata_dir"
        # Try alternative paths
        log_info "Searching for alternative testdata paths..."
        find "$BUF_PROJECT_ROOT" -type d -name "testdata" | head -5 | while read -r alt_dir; do
            log_info "Found testdata directory: $alt_dir"
        done
        return
    fi
    
    log_info "Test data directory found, scanning for YAML files..."
    local total_files=$(find "$testdata_dir" -name "*.yaml" | wc -l)
    log_info "Found $total_files YAML files in testdata directory"
    
    local count=0
    local yaml_files=()
    local proto_files=()
    mapfile -t yaml_files < <(find "$testdata_dir" -name "*.yaml")
    mapfile -t proto_files < <(find "$testdata_dir" -name "*.proto")
    
    log_info "YAML files to process: ${#yaml_files[@]}"
    log_info "Proto files to process: ${#proto_files[@]}"
    
    # Process YAML files
    for yaml_file in "${yaml_files[@]}"; do
        if [[ -z "$yaml_file" ]]; then
            continue
        fi
        
        # Get relative path from testdata root
        local rel_path="${yaml_file#$testdata_dir/}"
        local output_file="$output_dir/$rel_path"
        
        log_info "Processing YAML ($((count + 1))/${#yaml_files[@]}): $rel_path"
        
        # Create directory structure
        mkdir -p "$(dirname "$output_file")"
        
        # Convert to proto-sign format directly (skip copying original)
        local protosign_file="${output_file%.yaml}-protosign.yaml"
        if ! convert_to_protosign_format "$yaml_file" "$protosign_file"; then
            log_warn "Failed to convert: $yaml_file"
            continue
        fi
        
        ((count++))
        if (( count % 10 == 0 )); then
            log_info "Processed $count/${#yaml_files[@]} YAML configurations..."
        fi
    done
    
    # Process Proto files
    local proto_count=0
    for proto_file in "${proto_files[@]}"; do
        if [[ -z "$proto_file" ]]; then
            continue
        fi
        
        # Get relative path from testdata root
        local rel_path="${proto_file#$testdata_dir/}"
        local output_file="$output_dir/$rel_path"
        
        # Create directory structure
        mkdir -p "$(dirname "$output_file")"
        
        # Copy proto file
        if ! cp "$proto_file" "$output_file"; then
            log_warn "Failed to copy proto file: $proto_file"
            continue
        fi
        
        ((proto_count++))
        if (( proto_count % 20 == 0 )); then
            log_info "Processed $proto_count/${#proto_files[@]} proto files..."
        fi
    done
    
    log_info "Extracted $count YAML configuration files and $proto_count proto files"
}

# Convert buf.yaml format to proto-sign format
convert_to_protosign_format() {
    local input_file="$1"
    local output_file="$2"
    
    # Transform using yq
    {
        echo "# Generated from buf configuration"
        echo "# Original file: $(basename "$input_file")"
        echo "version: v1"
    } > "$output_file"
    
    # Check if input file exists and is readable
    if [[ ! -f "$input_file" ]]; then
        log_warn "Input file not found: $input_file"
        return 1
    fi
    
    # Extract breaking configuration
    if "$YQ_CMD" eval '.breaking' "$input_file" > /dev/null 2>&1; then
        echo "breaking:" >> "$output_file"
        
        # Extract use rules and categorize them
        if "$YQ_CMD" eval '.breaking.use[]' "$input_file" 2>/dev/null | head -1 > /dev/null; then
            local use_rules=$("$YQ_CMD" eval '.breaking.use[]' "$input_file" 2>/dev/null)
            # Check if these are categories (FILE, PACKAGE, WIRE, WIRE_JSON) or specific rules
            if echo "$use_rules" | grep -E '^(FILE|PACKAGE|WIRE|WIRE_JSON)$' > /dev/null; then
                echo "  use_categories:" >> "$output_file"
                echo "$use_rules" | sed 's/^/    - /' >> "$output_file"
            else
                echo "  use_rules:" >> "$output_file"
                echo "$use_rules" | sed 's/^/    - /' >> "$output_file"
            fi
        fi
        
        # Extract except rules
        local except_rules=$("$YQ_CMD" eval '.breaking.except[]' "$input_file" 2>/dev/null)
        if [[ -n "$except_rules" ]]; then
            echo "  except_rules:" >> "$output_file"
            echo "$except_rules" | sed 's/^/    - /' >> "$output_file"
        fi
        
        # Extract ignore paths
        local ignore_paths=$("$YQ_CMD" eval '.breaking.ignore[]' "$input_file" 2>/dev/null)
        if [[ -n "$ignore_paths" ]]; then
            echo "  ignore:" >> "$output_file"
            echo "$ignore_paths" | sed 's/^/    - /' >> "$output_file"
        fi
        
        # Extract ignore_unstable_packages
        local ignore_unstable=$("$YQ_CMD" eval '.breaking.ignore_unstable_packages' "$input_file" 2>/dev/null)
        if [[ "$ignore_unstable" == "true" || "$ignore_unstable" == "false" ]]; then
            echo "  ignore_unstable_packages: $ignore_unstable" >> "$output_file"
        fi
    fi
    
    log_info "Converted: $(basename "$input_file") -> $(basename "$output_file")"
}


# Note: Examples are manually maintained in compat-configs/examples/
# This script no longer auto-generates examples from buf project

# Generate summary report
generate_summary() {
    local summary_file="$EXTRACTED_CONFIGS_DIR/EXTRACTION_SUMMARY.md"
    local git_commit=""
    
    # Get git commit info from cloned repository
    if [[ -d "$BUF_PROJECT_ROOT/.git" ]]; then
        git_commit=$(cd "$BUF_PROJECT_ROOT" && git rev-parse --short HEAD 2>/dev/null || echo "unknown")
    fi
    
    cat > "$summary_file" << EOF
# Buf Configuration Extraction Summary

Generated on: $(date)
Buf git repository: $BUF_GIT_URL
Buf branch/commit: $BUF_BRANCH ($git_commit)
Extraction target: $EXTRACTED_CONFIGS_DIR

## Extracted Files

### Main Configuration
- proto-sign-main.yaml (converted from Buf main config)

### Test Data Configurations
- YAML files: $(find "$EXTRACTED_CONFIGS_DIR/testdata" -name "*.yaml" 2>/dev/null | wc -l || echo "0")
- Proto files: $(find "$EXTRACTED_CONFIGS_DIR/testdata" -name "*.proto" 2>/dev/null | wc -l || echo "0")
- Test directories: $(find "$EXTRACTED_CONFIGS_DIR/testdata" -type d -mindepth 2 2>/dev/null | wc -l || echo "0")

### Example Configurations  
Examples are manually maintained in compat-configs/examples/ (not auto-generated)

### Complete File Statistics
- Total files: $(find "$EXTRACTED_CONFIGS_DIR" -type f 2>/dev/null | wc -l || echo "0")
- YAML files (all): $(find "$EXTRACTED_CONFIGS_DIR" -name "*.yaml" 2>/dev/null | wc -l || echo "0") 
- Proto files (all): $(find "$EXTRACTED_CONFIGS_DIR" -name "*.proto" 2>/dev/null | wc -l || echo "0")
- Proto-sign configs: $(find "$EXTRACTED_CONFIGS_DIR" -name "*protosign.yaml" 2>/dev/null | wc -l || echo "0")

## Rule Categories Found

### Breaking Change Categories
EOF
    
    # Extract unique rule categories
    find "$EXTRACTED_CONFIGS_DIR" -name "*protosign.yaml" -exec grep -h "use_categories:" -A 20 {} \; 2>/dev/null | \
        grep "^    - " | sort -u >> "$summary_file" 2>/dev/null || true
    
    cat >> "$summary_file" << EOF

### Breaking Change Rules
EOF
    
    # Extract unique rule IDs
    find "$EXTRACTED_CONFIGS_DIR" -name "*protosign.yaml" -exec grep -h "use_rules:" -A 50 {} \; 2>/dev/null | \
        grep "^    - " | sort -u | head -20 >> "$summary_file" 2>/dev/null || true
    
    log_info "Generated summary report: $summary_file"
}

# Setup yq tool (download to temp if needed)
setup_yq() {
    YQ_CMD="yq"
    
    # Check if system yq exists and is correct version
    if command -v yq &> /dev/null; then
        local yq_version=$(yq --version 2>&1 || echo "unknown")
        if echo "$yq_version" | grep -i "mikefarah\|yq version" > /dev/null; then
            log_info "Using system mikefarah/yq (Go version)"
            return 0
        else
            log_warn "System yq is not mikefarah/yq version (detected: $yq_version)"
        fi
    else
        log_warn "No yq found in system PATH"
    fi
    
    log_info "Downloading mikefarah/yq to temporary directory..."
    
    # Detect architecture
    local arch=""
    case "$(uname -m)" in
        x86_64) arch="amd64" ;;
        aarch64|arm64) arch="arm64" ;;
        armv7l) arch="arm" ;;
        *) 
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    # Detect OS
    local os=""
    case "$(uname -s)" in
        Linux) os="linux" ;;
        Darwin) os="darwin" ;;
        *)
            log_error "Unsupported OS: $(uname -s)"
            exit 1
            ;;
    esac
    
    # Create temp dir for yq if not exists
    if [[ -z "${YQ_TEMP_DIR:-}" ]]; then
        YQ_TEMP_DIR=$(mktemp -d -t yq-XXXXXX)
        trap "cleanup_yq_temp" EXIT
    fi
    
    local yq_url="https://github.com/mikefarah/yq/releases/latest/download/yq_${os}_${arch}"
    local yq_path="$YQ_TEMP_DIR/yq"
    
    if ! wget -q -O "$yq_path" "$yq_url"; then
        log_error "Failed to download yq from $yq_url"
        exit 1
    fi
    
    chmod +x "$yq_path"
    YQ_CMD="$yq_path"
    
    # Verify downloaded yq works
    if ! "$YQ_CMD" --version >/dev/null 2>&1; then
        log_error "Downloaded yq is not working properly"
        exit 1
    fi
    
    log_info "Successfully downloaded and verified yq: $("$YQ_CMD" --version)"
}

# Cleanup temporary yq
cleanup_yq_temp() {
    if [[ -n "${YQ_TEMP_DIR:-}" && -d "$YQ_TEMP_DIR" ]]; then
        rm -rf "$YQ_TEMP_DIR"
    fi
}

# Check dependencies
check_dependencies() {
    # Check git availability
    if ! command -v git &> /dev/null; then
        log_error "git is required but not installed"
        exit 1
    fi
    
    # Check wget availability (needed for downloading yq)
    if ! command -v wget &> /dev/null; then
        log_error "wget is required but not installed"
        exit 1
    fi
    
    log_info "Dependencies check passed"
}

# Main execution
main() {
    log_info "Starting Buf configuration extraction..."
    
    check_dependencies
    setup_yq
    clone_buf_project
    setup_directories
    extract_main_config
    extract_testdata_configs
    generate_summary
    
    log_info "Extraction completed successfully!"
    log_info "Results available in: $EXTRACTED_CONFIGS_DIR"
    log_info "Review the summary: $EXTRACTED_CONFIGS_DIR/EXTRACTION_SUMMARY.md"
}

# Command line options
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [options]"
        echo ""
        echo "Extract and convert Buf configurations to proto-sign format"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --check        Check dependencies only"
        echo "  --branch BRANCH Set buf git branch (default: main)"
        echo ""
        echo "Environment Variables:"
        echo "  BUF_BRANCH     Override buf git branch (default: main)"
        echo ""
        echo "Requirements:"
        echo "  - git command line tool"
        echo "  - wget command line tool"
        echo "  - Internet connection to clone buf repository and download tools"
        echo ""
        echo "Note: yq (mikefarah/yq) will be automatically downloaded if not found"
        echo ""
        echo "Examples:"
        echo "  $0                    # Extract from main branch"
        echo "  $0 --branch v1.28.1   # Extract from specific tag"
        echo "  BUF_BRANCH=dev $0     # Extract from dev branch"
        exit 0
        ;;
    --check)
        check_dependencies
        log_info "Dependencies check passed"
        exit 0
        ;;
    --branch)
        BUF_BRANCH="$2"
        shift 2
        main "$@"
        ;;
    *)
        main "$@"
        ;;
esac