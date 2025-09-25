# 查找并修复所有 Rust 文件中的 BOM 问题
$rustFiles = Get-ChildItem -Path "src" -Filter "*.rs" -Recurse
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
$fixedCount = 0

Write-Host "正在检查所有 Rust 文件的 BOM 编码问题..." -ForegroundColor Cyan

foreach ($file in $rustFiles) {
    # 读取文件的前三个字节
    $bytes = Get-Content -Path $file.FullName -Encoding Byte -TotalCount 3
    
    # 检查是否有 BOM (0xEF 0xBB 0xBF)
    if (($bytes.Count -ge 3) -and ($bytes[0] -eq 0xEF) -and ($bytes[1] -eq 0xBB) -and ($bytes[2] -eq 0xBF)) {
        Write-Host "发现 BOM 字符在文件中: $($file.FullName)" -ForegroundColor Yellow
        
        # 读取全部内容并移除 BOM
        $content = Get-Content -Path $file.FullName -Raw
        
        # 重新保存文件，不带 BOM
        [System.IO.File]::WriteAllText($file.FullName, $content, $utf8NoBom)
        
        $fixedCount++
        Write-Host "✓ 已修复" -ForegroundColor Green
    }
}

if ($fixedCount -eq 0) {
    Write-Host "没有发现 BOM 问题" -ForegroundColor Green
} else {
    Write-Host "共修复了 $fixedCount 个文件的 BOM 编码问题" -ForegroundColor Green
}

Write-Host "`n现在尝试编译项目:" -ForegroundColor Cyan
cargo build