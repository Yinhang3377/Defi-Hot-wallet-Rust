# Find and fix BOM issues in all Rust files
$rustFiles = Get-ChildItem -Path "src" -Filter "*.rs" -Recurse
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
$fixedCount = 0

Write-Host "Checking all Rust files for BOM encoding issues..." -ForegroundColor Cyan

foreach ($file in $rustFiles) {
    # Read the first three bytes of the file
    $bytes = Get-Content -Path $file.FullName -Encoding Byte -TotalCount 3
    
    # Check for BOM (0xEF 0xBB 0xBF)
    if (($bytes.Count -ge 3) -and ($bytes[0] -eq 0xEF) -and ($bytes[1] -eq 0xBB) -and ($bytes[2] -eq 0xBF)) {
        Write-Host "Found BOM in file: $($file.FullName)" -ForegroundColor Yellow
        
        # Read the entire content and remove BOM
        $content = Get-Content -Path $file.FullName -Raw
        
        # Save the file without BOM
        [System.IO.File]::WriteAllText($file.FullName, $content, $utf8NoBom)
        
        $fixedCount++
        Write-Host "✓ Fixed" -ForegroundColor Green
    }
}

if ($fixedCount -eq 0) {
    Write-Host "No BOM issues found" -ForegroundColor Green
} else {
    Write-Host "Fixed BOM encoding in $fixedCount files" -ForegroundColor Green
}

Write-Host "`nNow trying to compile the project:" -ForegroundColor Cyan
cargo build