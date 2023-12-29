# todo deploy me as part of the CI

Output demo/demo.gif

Set FontSize 14
Set Width 900
Set Height 700

Set WindowBar Colorful
Set WindowBarSize 40

Type@10ms "cargo run --package solitaire --bin solitaire-tui --features=tui -- --seed=42"
Enter

# Deal animation takes ~2.8 seconds
Sleep 3s

Sleep 2s

Type @0.5s "s"
Space @1s
Type @0.5s "d"
Space @1s

Type @0.5s "d"
Type @0.5s "d"
Sleep 0.7s

Shift+A
Sleep 0.5s
Type @0.7s "a"
Space @1s

Shift+W
Sleep 0.5s
Shift+W
Sleep 0.7s

Shift+D
Sleep 0.5s
Type @0.5s "d"
Type @0.7s "d"
Space @1s

Shift+W
Sleep 0.5s
Shift+W
Sleep 1s

Shift+A
Sleep 0.5s
Shift+A
Sleep 0.5s
Shift+A
Sleep 0.5s
Shift+A
Sleep 0.7s
Space @1s

Type @0.7s "w"
Space @1s

Type @0.5s "d"
Shift+S
Sleep 0.5s
Type @0.7s "a"
Space @1s

Type @0.5s "d"
Type @0.7s "d"
Shift+A
Sleep 0.5s
Type @0.7s "a"
Space @1s

Sleep 2s
