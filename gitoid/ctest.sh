#!/usr/bin/env bash

# Variables for cbindgen.
CBINDGEN_CONFIG="./cbindgen.toml"

# Variables for the header file.
INCLUDE_DIR="./include"
HEADER_NAME="gitoid.h"

# Variables for the DLL/archive file.
LIB_DIR="../target/release"
LIB_NAME="gitoid"

# Variables for the test files.
TEST_SRC_FILE="./test/c/test.c"
TEST_EXE_FILE="./test/c_test"

echo "--------------------------------------------------"
echo "BUILDING LIBRARY"
cargo build -p "gitoid:0.4.0" --release --verbose

echo "--------------------------------------------------"
echo "GENERATING C HEADER FILE"
mkdir -p "${INCLUDE_DIR}"
cbindgen -v --clean -c "${CBINDGEN_CONFIG}" -o "${INCLUDE_DIR}/${HEADER_NAME}" --crate "gitoid"

echo "--------------------------------------------------"
echo "BUILDING C TEST FILE"
gcc --std=c99 -I"${INCLUDE_DIR}" -L"${LIB_DIR}" -o"${TEST_EXE_FILE}" "${TEST_SRC_FILE}" -l"${LIB_NAME}" 

echo "--------------------------------------------------"
echo "RUNNING C TEST FILE"
PATH="${LIB_DIR}:${PATH}" LD_LIBRARY_PATH="${LIB_DIR}" "${TEST_EXE_FILE}"

echo "--------------------------------------------------"
echo "DELETING C TEST FILE"
rm "${TEST_EXE_FILE}"

