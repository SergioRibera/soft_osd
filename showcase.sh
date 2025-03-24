#!/usr/bin/env sh

sosd="$PWD/target/release/sosd"
ts="3s"
sleep 2s

$sosd -b "#fff" -f "#000" notification -m "" -t "Hello, world!" -d "This is a showcase of the sosd notification command."
sleep $ts

$sosd -b "#ff6961" -f "#000" notification -m "" -t "Oh no!" -d "This is a critical notification."
sleep $ts

$sosd -b "#5ba7f5" notification -t "Relax!" -d "This is a low urgency notification."
sleep $ts

$sosd slider -m "󰁼" -v "70"
sleep $ts

$sosd -b "#ed6896" -f "#fff" slider -m "󰁼" -v "35"
sleep $ts

$sosd -b "#01D354" -f "#fff" notification -m "" -t "Like a Stone - Playing" -d "Audioslave"
sleep $ts

$sosd -b "#f5b95b" -f "#000" notification -m "󰌐" -t "Numlock enabled"
sleep $ts

$sosd -b "#6F84D4" -f "#fff" notification -m "" -t "#🤓-general - RustLangES" -d "you every time we talk about linux 🤣"
sleep $ts
