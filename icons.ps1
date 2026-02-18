
# read shell script, remove '()', replace '$1' to '$($args[0])', pipe to powershell
Get-Content icons.sh | ForEach-Object { $_ -replace '\(\)' -replace "\$\d", {"`$(`$args[$([int]$_.Value.Substring(1)-1)])"} } | powershell
