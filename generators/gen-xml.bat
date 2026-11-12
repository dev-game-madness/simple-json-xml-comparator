@echo off
rem Writes xml-YYYY-MM-DD-gen-1.xml and xml-YYYY-MM-DD-gen-2.xml next to this bat.
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
<?xml version="1.0" encoding="UTF-8"?>
<root>
  <document>
    <id>DOC-100</id>
    <title>Сравнение поставок</title>
    <created>2026-11-12</created>
  </document>
  <customer>
    <name>Анна Петрова</name>
    <city>Казань</city>
    <vip>true</vip>
  </customer>
  <order>
    <number>4812</number>
    <paid>false</paid>
    <currency>RUB</currency>
    <total>1540.5</total>
    <note>оплата по договору</note>
  </order>
  <items>
    <item>
      <sku>A-10</sku>
      <name>Кабель</name>
      <qty>2</qty>
      <price>120.0</price>
      <inStock>true</inStock>
      <meta>
        <color>чёрный</color>
        <length>1 м</length>
      </meta>
    </item>
    <item>
      <sku>B-22</sku>
      <name>Адаптер</name>
      <qty>1</qty>
      <price>890.5</price>
      <inStock>false</inStock>
      <meta>
        <color>белый</color>
        <length>0.2 м</length>
      </meta>
    </item>
    <item>
      <sku>C-07</sku>
      <name>Розетка</name>
      <qty>4</qty>
      <price>75.0</price>
      <inStock>true</inStock>
      <meta>
        <color>серый</color>
        <length>стандарт</length>
      </meta>
    </item>
  </items>
  <flags>
    <archived>false</archived>
    <priority>normal</priority>
    <tags>
      <tag>
        <code>P1</code>
        <label>срочно</label>
      </tag>
      <tag>
        <code>N2</code>
        <label>склад</label>
      </tag>
    </tags>
  </flags>
</root>
'@

$right = @'
<?xml version="1.0" encoding="UTF-8"?>
<root>
  <document>
    <id>DOC-100</id>
    <title>Сверка отгрузок</title>
    <created>2026-11-12</created>
  </document>
  <customer>
    <name>Борис Смирнов</name>
    <city>Москва</city>
    <vip>true</vip>
  </customer>
  <order>
    <number>4812</number>
    <paid>false</paid>
    <currency>RUB</currency>
    <total>2100.0</total>
    <note>оплата онлайн</note>
  </order>
  <items>
    <item>
      <sku>A-10</sku>
      <name>Кабель HDMI</name>
      <qty>2</qty>
      <price>150.0</price>
      <inStock>true</inStock>
      <meta>
        <color>чёрный</color>
        <length>2 м</length>
      </meta>
    </item>
    <item>
      <sku>B-22</sku>
      <name>Адаптер Type-C</name>
      <qty>1</qty>
      <price>640.0</price>
      <inStock>true</inStock>
      <meta>
        <color>белый</color>
        <length>0.5 м</length>
      </meta>
    </item>
    <item>
      <sku>C-07</sku>
      <name>Выключатель</name>
      <qty>6</qty>
      <price>75.0</price>
      <inStock>true</inStock>
      <meta>
        <color>бежевый</color>
        <length>компакт</length>
      </meta>
    </item>
  </items>
  <flags>
    <archived>false</archived>
    <priority>high</priority>
    <tags>
      <tag>
        <code>P1</code>
        <label>срочно</label>
      </tag>
      <tag>
        <code>N3</code>
        <label>витрина</label>
      </tag>
    </tags>
  </flags>
</root>
'@

Write-Doc "xml-$date-gen-1.xml" $left
Write-Doc "xml-$date-gen-2.xml" $right
