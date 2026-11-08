$queries = @(
    @{ q = 't'; mode = 'code'; label = 't (typing 1 char)' },
    @{ q = 'tr'; mode = 'code'; label = 'tr (typing 2 chars)' },
    @{ q = 'tron'; mode = 'code'; label = 'tron (identifier)' },
    @{ q = 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t'; mode = 'code'; label = 'TRON Base58Check' },
    @{ q = 'DE89370400440532013000'; mode = 'code'; label = 'IBAN Mod-97' },
    @{ q = 'OpenAI'; mode = 'global'; label = 'OpenAI (Web)' },
    @{ q = 'Anthropic'; mode = 'global'; label = 'Anthropic (Web)' },
    @{ q = 'Google'; mode = 'global'; label = 'Google (Web)' },
    @{ q = 'deepseek v3 architecture and moe'; mode = 'global'; label = 'DeepSeek Architecture (Web)' }
)

Write-Host ""
Write-Host "===================================================================================================================="
Write-Host " FurlPay Search Progressive SSE Live Latency & Byte Telemetry Matrix"
Write-Host "===================================================================================================================="
Write-Host ("{0,-28} | {1,-6} | {2,-4} | {3,8} | {4,10} | {5,10} | {6}" -f "Query Label", "Mode", "HTTP", "Bytes", "TTFB", "Total Time", "Top Match / Entity")
Write-Host "--------------------------------------------------------------------------------------------------------------------"

foreach ($item in $queries) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $encoded = [System.Uri]::EscapeDataString($item.q)
    $url = "http://127.0.0.1:3000/api/v1/search/stream?q=$encoded&mode=$($item.mode)"
    
    $req = [System.Net.HttpWebRequest]::Create($url)
    $req.Timeout = 12000
    try {
        $resp = $req.GetResponse()
        $ttfb = $sw.ElapsedMilliseconds
        $stream = $resp.GetResponseStream()
        $reader = New-Object System.IO.StreamReader($stream)
        $text = $reader.ReadToEnd()
        $sw.Stop()
        $totalMs = $sw.ElapsedMilliseconds
        $bytes = [System.Text.Encoding]::UTF8.GetByteCount($text)
        $statusCode = [int]$resp.StatusCode

        $topMatch = "N/A"
        if ($text -match '"title":"([^"]+)"') {
            $topMatch = $matches[1]
        } elseif ($text -match '"query_class":"([^"]+)"') {
            $topMatch = $matches[1]
        }

        Write-Host ("{0,-28} | {1,-6} | {2,-4} | {3,6} B | {4,8} ms | {5,8} ms | {6}" -f $item.label, $item.mode, $statusCode, $bytes, $ttfb, $totalMs, $topMatch)
    } catch {
        Write-Host ("{0,-28} | {1,-6} | ERR: {2}" -f $item.label, $item.mode, $_.Exception.Message)
    }
}
Write-Host "===================================================================================================================="
Write-Host ""
