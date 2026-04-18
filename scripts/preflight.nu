#!/usr/bin/env nu
# preflight.nu — doob environment validation

def check [label: string, pass: bool, detail: string = ""] {
    if $pass {
        print $"[ok]   ($label)"
    } else if $detail != "" {
        print $"[fail] ($label) — ($detail)"
    } else {
        print $"[fail] ($label)"
    }
    $pass
}

print "=== doob preflight ==="

let db_dir = ($env.HOME | path join ".ctx" "doob" "db")
let db_exists = ($db_dir | path exists)

let results = [
    (check "cargo on PATH" (which cargo | length) > 0),
    (check "just on PATH" (which just | length) > 0),
    (check "doob on PATH" (which doob | length) > 0 "run: cargo install --path ."),
    (check "doobdash on PATH" (which doobdash | length) > 0 "run: cargo install --path crates/doobdash"),
    (check "op on PATH" (which op | length) > 0),
    (check "1Password authed" (do { op account list } | complete | get exit_code) == 0),
    (check "SurrealKV db dir exists" $db_exists $"($db_dir) — will be created on first run"),
    (check "git repo clean" (do { git status --porcelain } | complete | get stdout | str trim | is-empty)),
]

let failed = $results | where { |r| not $r } | length
let total = $results | length

print ""
if $failed == 0 {
    print $"preflight passed ($total)/($total)"
} else {
    print $"preflight ($total - $failed)/($total) — ($failed) check(s) failed"
}
