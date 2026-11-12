@echo off
rem Writes json-YYYY-MM-DD-gen-1.json and json-YYYY-MM-DD-gen-2.json next to this bat.
rem String values contain \" and embedded objects so quote-unescape has work to do.
setlocal EnableExtensions
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "$dir='%~dp0'; $raw=[Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes('%~f0')); $mark='###POWER'+'SHELL###'; $i=$raw.IndexOf($mark); if($i -lt 0){ throw 'marker missing' }; Invoke-Expression $raw.Substring($i+$mark.Length)"
exit /b %ERRORLEVEL%
###POWERSHELL###
$ErrorActionPreference = 'Stop'
$utf8 = New-Object System.Text.UTF8Encoding $false
$date = Get-Date -Format 'yyyy-MM-dd'
$folder = $dir.TrimEnd('\')

function Write-Doc([string]$name, [string]$text) {
  $clean = $text.Trim().Replace("`r`n", "`n").Replace("`r", "`n")
  if (-not $clean.EndsWith("`n")) { $clean += "`n" }
  [System.IO.File]::WriteAllText((Join-Path $folder $name), $clean, $utf8)
}

$left = @'
{
  "document": {
    "id": "DOC-100",
    "title": "Сравнение поставок",
    "created": "2026-11-12"
  },
  "customer": "{\"name\": \"Анна Петрова\", \"city\": \"Казань\", \"vip\": true}",
  "order": {
    "number": 4812,
    "paid": false,
    "currency": "RUB",
    "total": 1540.5,
    "note": "{\"method\": \"счёт\", \"comment\": \"оплата по договору\"}"
  },
  "items": [
    {
      "sku": "A-10",
      "name": "Кабель",
      "qty": 2,
      "price": 120.0,
      "inStock": true,
      "meta": "{\"color\": \"чёрный\", \"length\": \"1 м\"}"
    },
    {
      "sku": "B-22",
      "name": "Адаптер",
      "qty": 1,
      "price": 890.5,
      "inStock": false,
      "meta": "{\"color\": \"белый\", \"length\": \"0.2 м\"}"
    },
    {
      "sku": "C-07",
      "name": "Розетка",
      "qty": 4,
      "price": 75.0,
      "inStock": true,
      "meta": "{\"color\": \"серый\", \"length\": \"стандарт\"}"
    }
  ],
  "flags": {
    "archived": false,
    "priority": "normal",
    "tags": "[{\"code\": \"P1\", \"label\": \"срочно\"}, {\"code\": \"N2\", \"label\": \"склад\"}]"
  }
}
'@

$right = @'
{
  "document": {
    "id": "DOC-100",
    "title": "Сверка отгрузок",
    "created": "2026-11-12"
  },
  "customer": "{\"name\": \"Борис Смирнов\", \"city\": \"Москва\", \"vip\": true}",
  "order": {
    "number": 5099,
    "paid": false,
    "currency": "RUB",
    "total": 2100.0,
    "note": "{\"method\": \"карта\", \"comment\": \"оплата онлайн\"}"
  },
  "items": [
    {
      "sku": "A-10",
      "name": "Кабель HDMI",
      "qty": 2,
      "price": 150.0,
      "inStock": true,
      "meta": "{\"color\": \"чёрный\", \"length\": \"2 м\"}"
    },
    {
      "sku": "B-22",
      "name": "Адаптер",
      "qty": 1,
      "price": 640.0,
      "inStock": true,
      "meta": "{\"color\": \"белый\", \"length\": \"0.2 м\"}"
    },
    {
      "sku": "C-07",
      "name": "Выключатель",
      "qty": 6,
      "price": 75.0,
      "inStock": true,
      "meta": "{\"color\": \"бежевый\", \"length\": \"стандарт\"}"
    }
  ],
  "flags": {
    "archived": false,
    "priority": "high",
    "tags": "[{\"code\": \"P1\", \"label\": \"срочно\"}, {\"code\": \"N3\", \"label\": \"витрина\"}]"
  }
}
'@

Write-Doc "json-$date-gen-1.json" $left
Write-Doc "json-$date-gen-2.json" $right
