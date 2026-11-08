# FurlPay Local Retrieval Quality Regression Test & Benchmark Suite 2026
# Tests all 11 required queries across 30 iterations each and reports latency percentiles

$baseUrl = "http://127.0.0.1:3000/api/v1/search/stream"
$queries = @(
    @{ name = "Single character (t)"; query = "t"; expected_top = "crates/furlpay-core/src/index/trigram.rs"; check_kind = "prefix" },
    @{ name = "Short token prefix (tr)"; query = "tr"; expected_top = "crates/furlpay-core/src/index/trigram.rs"; check_kind = "prefix" },
    @{ name = "Token prefix (tro)"; query = "tro"; expected_top = "blockchain/tron.ts"; check_kind = "prefix" },
    @{ name = "Full token (tron)"; query = "tron"; expected_top = "blockchain/tron.ts"; check_kind = "exact_stem" },
    @{ name = "Identifier with extension (tron.ts)"; query = "tron.ts"; expected_top = "blockchain/tron.ts"; check_kind = "exact_path" },
    @{ name = "Core component (trigram)"; query = "trigram"; expected_top = "crates/furlpay-core/src/index/trigram.rs"; check_kind = "exact_stem" },
    @{ name = "Unfinished component (treesitter)"; query = "treesitter"; expected_top = "crates/furlpay-core/src/ast/treesitter.rs"; check_kind = "stub_deprioritized" },
    @{ name = "Explicit stub search (treesitter stub)"; query = "treesitter stub"; expected_top = "crates/furlpay-core/src/ast/treesitter.rs"; check_kind = "stub_prioritized" },
    @{ name = "Explicit TODO search (TODO treesitter)"; query = "TODO treesitter"; expected_top = "crates/furlpay-core/src/ast/treesitter.rs"; check_kind = "stub_prioritized" },
    @{ name = "Full relative path (blockchain/tron.ts)"; query = "blockchain/tron.ts"; expected_top = "blockchain/tron.ts"; check_kind = "exact_path" },
    @{ name = "TRON USDT address (TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t)"; query = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"; expected_top = "blockchain://tron"; check_kind = "cryptographic_wallet" }
)

Write-Host "=====================================================================" -ForegroundColor Cyan
Write-Host " FurlPay Local Retrieval Quality & Verification Test Suite (11 Queries)" -ForegroundColor Cyan
Write-Host "=====================================================================" -ForegroundColor Cyan

$resultsReport = @()

foreach ($qObj in $queries) {
    $qName = $qObj.name
    $qStr = [System.Uri]::EscapeDataString($qObj.query)
    $url = "$baseUrl?q=$qStr&mode=code"
    
    # 1. Functional Verification
    $rawOutput = & curl.exe -s -N --max-time 5 $url
    
    $events = @{}
    $lines = $rawOutput -split "`n"
    $currentEvent = ""
    foreach ($line in $lines) {
        $line = $line.Trim()
        if ($line.StartsWith("event: ")) {
            $currentEvent = $line.Substring(7).Trim()
        } elseif ($line.StartsWith("data: ") -and $currentEvent -ne "") {
            $dataStr = $line.Substring(6).Trim()
            $events[$currentEvent] = $dataStr
        }
    }
    
    $completedJson = $events["search.completed"] | ConvertFrom-Json
    $resultsJson = $events["search.results"] | ConvertFrom-Json
    $verifJson = $events["search.verification"] | ConvertFrom-Json
    $classJson = $events["search.classified"] | ConvertFrom-Json
    
    $topResult = if ($resultsJson.results.Count -gt 0) { $resultsJson.results[0] } else { $null }
    $topFile = if ($topResult) { $topResult.file_path } else { "NONE" }
    $topScore = if ($topResult) { $topResult.score } else { 0.0 }
    $topVerifKind = if ($topResult) { $topResult.verification_kind } else { "NONE" }
    $topStatus = if ($topResult) { $topResult.implementation_status } else { "NONE" }
    $topReason = if ($topResult) { $topResult.ranking_reason } else { "NONE" }
    
    Write-Host "`nQuery: $($qObj.query) ($qName)" -ForegroundColor Yellow
    Write-Host "  Top File: $topFile (Score: $topScore, Status: $topStatus, Verif: $topVerifKind)" -ForegroundColor White
    Write-Host "  Reason: $topReason" -ForegroundColor Gray
    Write-Host "  Total Results: $($completedJson.total_results), Latency: $($completedJson.elapsed_ms) ms" -ForegroundColor Gray

    # 2. Benchmark 30 iterations
    $latencies = @()
    $ttfrs = @()
    for ($i = 0; $i -lt 30; $i++) {
        $iterRaw = & curl.exe -s -N --max-time 5 $url
        $iterLines = $iterRaw -split "`n"
        $iterComp = $null
        $iterEv = ""
        foreach ($il in $iterLines) {
            $il = $il.Trim()
            if ($il.StartsWith("event: ")) { $iterEv = $il.Substring(7).Trim() }
            elseif ($il.StartsWith("data: ") -and $iterEv -eq "search.completed") {
                $iterComp = $il.Substring(6).Trim() | ConvertFrom-Json
            }
        }
        if ($iterComp) {
            $latencies += $iterComp.elapsed_ms
            if ($iterComp.telemetry -and $iterComp.telemetry.first_result_latency_us) {
                $ttfrs += ($iterComp.telemetry.first_result_latency_us / 1000.0)
            }
        }
    }
    
    $sortedL = $latencies | Sort-Object
    $count = $sortedL.Count
    $minL = $sortedL[0]
    $maxL = $sortedL[$count - 1]
    $medL = $sortedL[[Math]::Floor($count * 0.50)]
    $p90L = $sortedL[[Math]::Floor($count * 0.90)]
    $p95L = $sortedL[[Math]::Floor($count * 0.95)]
    
    $sortedT = $ttfrs | Sort-Object
    $medT = if ($sortedT.Count -gt 0) { $sortedT[[Math]::Floor($sortedT.Count * 0.50)] } else { 0.0 }

    Write-Host "  Benchmark (30 runs): Min: $minL ms | Med: $medL ms | p90: $p90L ms | p95: $p95L ms | Max: $maxL ms | TTFR: $medT ms" -ForegroundColor Green

    $resultsReport += [PSCustomObject]@{
        Query = $qObj.query
        Category = $qName
        TopFile = $topFile
        Score = [Math]::Round($topScore, 3)
        Status = $topStatus
        VerificationKind = $topVerifKind
        TTFR_ms = [Math]::Round($medT, 3)
        Median_ms = [Math]::Round($medL, 3)
        P90_ms = [Math]::Round($p90L, 3)
        P95_ms = [Math]::Round($p95L, 3)
    }
}

Write-Host "`n=====================================================================" -ForegroundColor Cyan
Write-Host " Benchmark Summary Table" -ForegroundColor Cyan
Write-Host "=====================================================================" -ForegroundColor Cyan
$resultsReport | Format-Table -AutoSize
