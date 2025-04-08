#!/bin/bash

# remove python3 from apps/c/
rm -rf apps/c/python3

# Clone the repository
echo "Cloning the repository..."
git clone https://github.com/syswonder/RuxOS_Test.git apps/c/python3

# Check if cloning was successful
if [ $? -ne 0 ]; then
    echo "Failed to clone the repository."
    exit 1
fi

# Compile and run the Python test script using make
echo "Compiling and running the Python test script..."
make A=apps/c/python3 ARCH=aarch64 LOG=error V9P=y NET=y SMP=4 MUSL=y run > script.log 2>&1

# Check if the make command was successful
if [ $? -ne 0 ]; then
    echo "Failed to compile and run the Python test script."
    exit 1
fi

# Define the sentences to check in the log file
declare -a sentences=(
    "Finished test_wait4.py"
    "Finished test_mmap.py"
    # Add more sentences as needed
)

# Check if the sentences exist in the log file
echo "Checking the log file for specific sentences..."
missing_sentences=()
for sentence in "${sentences[@]}"; do
    if ! grep -q "$sentence" script.log; then
        missing_sentences+=("$sentence")
    fi
done

# Output the results
if [ ${#missing_sentences[@]} -eq 0 ]; then
    echo "All specified sentences were found in the log file."
else
    echo "The following sentences were NOT found in the log file:"
    for missing in "${missing_sentences[@]}"; do
        echo "- $missing"
    done
    exit 1
fi
