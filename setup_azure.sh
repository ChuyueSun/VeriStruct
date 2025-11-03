#!/bin/bash
# Quick setup script for Azure OpenAI API

echo "🚀 Azure OpenAI API Setup"
echo "=========================="
echo ""

# Check if openai package is installed
echo "📦 Checking dependencies..."
if python3 -c "import openai" 2>/dev/null; then
    echo "✅ openai package is installed"
    python3 -c "import openai; print(f'   Version: {openai.__version__}')"
else
    echo "⚠️  openai package not found"
    echo "   Installing openai package..."
    pip install openai
fi

echo ""
echo "🔑 API Key Setup"
echo "==============="
echo ""
echo "You need to add your Azure OpenAI API key. Choose one option:"
echo ""
echo "Option 1: Set environment variable (recommended for testing)"
echo "  export AZURE_OPENAI_API_KEY='your-api-key-here'"
echo ""
echo "Option 2: Edit config file directly"
echo "  Edit: src/configs/config-azure.json"
echo "  Replace: YOUR_API_KEY_HERE with your actual key"
echo ""
echo "To get your API key:"
echo "1. Go to: https://portal.azure.com"
echo "2. Navigate to: Azure OpenAI → tacasproject-resource"
echo "3. Go to: Deployments + Endpoints"
echo "4. Copy your API key"
echo ""
echo "🧪 Test Your Setup"
echo "=================="
echo ""
echo "After setting your API key, run:"
echo "  python3 test_azure_api.py"
echo ""
echo "For multi-turn conversation test:"
echo "  python3 test_azure_multi_turn.py"
echo ""
echo "📚 For detailed setup instructions, see: AZURE_API_SETUP.md"
echo ""
