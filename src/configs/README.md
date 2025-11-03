# Configuration Setup

This directory contains configuration files for VerusAgent. The actual configuration files are ignored by git to prevent exposing API keys.

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
- `timeout_seconds`: Timeout for verification attempts (default: 300)
- `enable_cache`: Enable LLM response caching (default: true)
- `cache_dir`: Directory for cache files (default: "llm_cache")

### Path Settings

- `verus_path`: Path to Verus executable
- `benchmark_dir`: Directory containing benchmark files
- `output_dir`: Directory for output files

### Execution Settings

- `max_iterations`: Maximum workflow iterations (default: 10)
- `max_repair_attempts`: Maximum repair attempts per error (default: 5)
- `parallel_executions`: Number of parallel benchmark executions (default: 4)

## Example Configurations

### For Azure OpenAI (Recommended)
```bash
cp config.json.template config-azure.json
# Edit config-azure.json with your Azure credentials
```

### For OpenAI
```bash
cp config.json.template config-oai.json
# Edit config-oai.json with your OpenAI API key
```

### For Anthropic Claude
```bash
cp config.json.template config-anthropic.json
# Edit config-anthropic.json with your Anthropic API key
```

## Security Notes

⚠️ **IMPORTANT**:
- Never commit files containing actual API keys
- All `config*.json` files (except `.template`) are in `.gitignore`
- Keep your API keys secure and rotate them regularly
- Use environment variables for additional security if needed

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
