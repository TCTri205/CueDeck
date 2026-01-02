# Test MCP read_context with filters
# Usage: .\test_mcp_filters.ps1

Write-Host "🧪 Testing MCP Server - Search Filters" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Set workspace environment variable
$env:CUE_WORKSPACE = "D:\Projects_IT\CueDeck"

# Start MCP server in background
Write-Host "Starting MCP server..." -ForegroundColor Yellow
$mcpProcess = Start-Process -FilePath "cue" -ArgumentList "mcp" -NoNewWindow -PassThru -RedirectStandardInput "mcp_input.txt" -RedirectStandardOutput "mcp_output.txt" -RedirectStandardError "mcp_error.txt"

Start-Sleep -Seconds 2

# Test 1: Filter by tags
Write-Host "Test 1: Filter by tags (auth, security)" -ForegroundColor Green
$request1 = @{
    jsonrpc = "2.0"
    id = 1
    method = "tools/call"
    params = @{
        name = "read_context"
        arguments = @{
            query = "authentication"
            limit = 5
            filters = @{
                tags = @("auth", "security")
            }
        }
    }
} | ConvertTo-Json -Depth 10

Write-Host $request1 -ForegroundColor Gray
$request1 | Out-File -FilePath "mcp_input.txt" -Encoding UTF8
Start-Sleep -Seconds 1
Get-Content "mcp_output.txt" | Write-Host -ForegroundColor White
Write-Host ""

# Test 2: Filter by priority
Write-Host "Test 2: Filter by priority (high)" -ForegroundColor Green
$request2 = @{
    jsonrpc = "2.0"
    id = 2
    method = "tools/call"
    params = @{
        name = "read_context"
        arguments = @{
            query = "database"
            limit = 5
            filters = @{
                priority = "high"
            }
        }
    }
} | ConvertTo-Json -Depth 10

Write-Host $request2 -ForegroundColor Gray
$request2 | Out-File -FilePath "mcp_input.txt" -Encoding UTF8 -Append
Start-Sleep -Seconds 1
Get-Content "mcp_output.txt" | Write-Host -ForegroundColor White
Write-Host ""

# Test 3: Combined filters
Write-Host "Test 3: Combined filters (tags + priority)" -ForegroundColor Green
$request3 = @{
    jsonrpc = "2.0"
    id = 3
    method = "tools/call"
    params = @{
        name = "read_context"
        arguments = @{
            query = "api"
            limit = 5
            filters = @{
                tags = @("backend", "api")
                priority = "high"
            }
        }
    }
} | ConvertTo-Json -Depth 10

Write-Host $request3 -ForegroundColor Gray
$request3 | Out-File -FilePath "mcp_input.txt" -Encoding UTF8 -Append
Start-Sleep -Seconds 1
Get-Content "mcp_output.txt" | Write-Host -ForegroundColor White

# Cleanup
Write-Host ""
Write-Host "Stopping MCP server..." -ForegroundColor Yellow
Stop-Process -Id $mcpProcess.Id -Force
Remove-Item "mcp_input.txt", "mcp_output.txt", "mcp_error.txt" -ErrorAction SilentlyContinue

Write-Host "✅ Test completed!" -ForegroundColor Green
