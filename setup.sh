#!/bin/bash

# Name of the .env file
ENV_FILE=".env"
HOME="pyano_home"
MODEL_HOME="pyano_home/models"
CONFIG_HOME="pyano_home/configs"

# Check if the file already exists
if [ -f "$ENV_FILE" ]; then
  echo "$ENV_FILE already exists. Do you want to overwrite it? (y/n)"
  read -r response
  if [[ "$response" != "y" ]]; then
    echo "Exiting without creating .env file."
    exit 1
  fi
fi

# Write environment variables to the file
cat > "$ENV_FILE" <<EOL
# Environment variables
HOME=pyano_home
MODEL_HOME=pyano_home/models
CONFIG_HOME=pyano_home/configs
EOL

echo "$ENV_FILE has been created."

# Check if directories exist, if not create them
for dir in "$HOME" "$MODEL_HOME" "$CONFIG_HOME"; do
    if [ ! -d "$dir" ]; then
        echo "Directory $dir does not exist. Creating it."
        mkdir -p "$dir"
    else
        echo "Directory $dir already exists."
    fi
done

# Copy all JSON files from examples/configs to CONFIG_HOME
EXAMPLES_CONFIGS="examples/configs"
if [ -d "$EXAMPLES_CONFIGS" ]; then
    cp "$EXAMPLES_CONFIGS"/*.json "$CONFIG_HOME/"
    echo "Copied all JSON files from $EXAMPLES_CONFIGS to $CONFIG_HOME."
else
    echo "Directory $EXAMPLES_CONFIGS does not exist. No files copied."
fi

