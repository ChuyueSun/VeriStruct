# Configuration Setup

This directory contains configuration files for VeriStruct. The actual configuration files are ignored by git to prevent exposing API keys.

## Quick Start

1. **Copy the template:**

   ```bash
   cp config.json.template config.json
   ```

2. **Edit `config.json` with your API credentials:**
   - For Azure OpenAI: Fill in `aoai_*` fields
   - For OpenAI: Fill in `openai_*` fields
   - For Anthropic: Fill in `anthropic_*` fields
   - For DeepSeek: Fill in `deepseek_*` fields

3. **Configure paths:**
   - Set `verus_path` to your Verus installation
   - Adjust `benchmark_dir` and `output_dir` as needed

## Configuration Options

### API Settings

#### Azure OpenAI

```json
{
  "aoai_api_key": "your-azure-api-key",
  "aoai_generation_model": "gpt-4",
  "aoai_api_base": "https://your-resource.openai.azure.com/",
  "aoai_api_version": "2023-05-15",
  "aoai_deployment_name": "your-deployment-name"
}
```

#### OpenAI

```json
{
  "openai_api_key": "sk-...",
  "openai_model": "gpt-4"
}
```

#### Anthropic Claude

```json
{
  "anthropic_api_key": "sk-ant-...",
  "anthropic_model": "claude-3-sonnet-20240229"
}
```

#### DeepSeek

```json
{
  "deepseek_api_key": "your-deepseek-key",
  "deepseek_api_base": "https://api.deepseek.com"
}
```

### System Settings

- `max_retries`: Number of retry attempts for failed API calls (default: 3)
- `repair_timeout`: Maximum time per repair attempt in seconds (default: 120)
- `repair_llm_timeout`: LLM call warning threshold in seconds (default: 60)
- `slow_repair_threshold`: Slow repair warning threshold in seconds (default: 30)
- `max_repair_retries`: Number of retry attempts on timeout (default: 1)
- `enable_cache`: Enable LLM response caching (default: true)
- `cache_dir`: Directory for cache files (default: "llm_cache")

### Path Settings

- `verus_path`: Path to Verus executable (optional, will use system PATH if not specified)
- `benchmark_dir`: Directory containing benchmark files (default: "benchmarks-complete")
- `output_dir`: Directory for output files (default: "output")

### Execution Settings

- `max_iterations`: Maximum workflow iterations (default: 10)
- `max_repair_attempts`: Maximum repair attempts per error (default: 5)
- `num_repair_rounds`: Number of repair rounds to attempt (configurable via CLI)

## Current Configurations

### Available

- **config-azure.json** - Azure OpenAI configuration (currently set up)
- **config.json.template** - Template for creating new configurations

### Creating Additional Configurations

#### For Azure OpenAI

```bash
# Already configured in config-azure.json
# Edit config-azure.json to update your Azure credentials
```

#### For OpenAI

```bash
cp config.json.template config-oai.json
# Edit config-oai.json with your OpenAI API key
```

#### For Anthropic Claude

```bash
cp config.json.template config-anthropic.json
# Edit config-anthropic.json with your Anthropic API key
```

#### For DeepSeek

```bash
cp config.json.template config-deepseek.json
# Edit config-deepseek.json with your DeepSeek API key
```

## 🔒 Security Notes

⚠️ **IMPORTANT - API Key Protection**:

✅ **Already Protected:**

- All `config*.json` files (except `.template`) are automatically ignored by git
- Your API keys in `config-azure.json` will **NEVER** be committed to the repository
- The `.gitignore` file ensures these files stay local only

⚠️ **Best Practices:**

- Never manually add config files to git (don't use `git add -f`)
- Never commit files containing actual API keys
- Keep your API keys secure and rotate them regularly
- Consider using environment variables for additional security:

```bash
# Set environment variable
export AZURE_OPENAI_API_KEY="your-key-here"

# Reference in config.json
{
  "aoai_api_key": "${AZURE_OPENAI_API_KEY}"
}
```

## Troubleshooting

**Config file not found:**

- Ensure you've copied the template to `config.json`
- Check that the file is in `src/configs/` directory

**API authentication errors:**

- Verify your API key is correct
- Check API endpoint URLs are valid
- Ensure your API subscription is active

**Path errors:**

- Verify Verus is installed and `verus_path` is correct
- Check that benchmark and output directories exist
