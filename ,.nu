const WORKDIR = path self .
const CFG = path self ,.toml
const GW = path self gateway.toml
const CHAT = path self chat.toml

export def workdir [] {
    $WORKDIR
}

def wait-cmd [action -i: duration = 1sec  -t: string='waiting'] {
    mut time = 0
    loop {
        print -e $"(ansi dark_gray)($t) (ansi dark_gray_italic)($i * $time)(ansi reset)"
        let c = do --ignore-errors $action | complete | get exit_code
        if ($c == 0) { break }
        sleep $i
        $time = $time + 1
    }
}

module pg {
    export def cli [query? --db:string = 'chat'] {
        let q = $in
        let q = if ($q | is-empty) { $query } else { $q }
        let cfg = open $CHAT | get database
        let db = $db | default $cfg.db
        let cmd = $"
            INSTALL postgres;
            LOAD postgres;
            ATTACH 'dbname=($db) user=($cfg.user) host=127.0.0.1 port=($cfg.port) password=($cfg.passwd)' AS ($db) \(TYPE postgres\);
            USE ($db)
        "
        if ($q | is-empty) {
            duckdb -cmd $cmd
        } else {
            $q | duckdb -cmd $cmd
        }
    }


    export def start [
        --dry-run
    ] {
        let cfg = open $CHAT | get database
        let image = 'postgres:17'
        mut args = [run -d --name chat_db]
        let ports = {
            $cfg.port: 5432
        }
        for i in ($ports | transpose k v) {
            $args ++= [-p $"($i.k):($i.v)"]
        }
        let envs = {
            POSTGRES_DB: $cfg.db
            POSTGRES_USER: $cfg.user
            POSTGRES_PASSWORD: $cfg.passwd
        }
        for i in ($envs | transpose k v) {
            $args ++= [-e $"($i.k)=($i.v)"]
        }
        $args ++= [-v $"($WORKDIR)/data/postgres/data:/var/lib/postgresql/data"]
        $args ++= [$image]
        if $dry_run {
            print $"($env.CNTRCTL) ($args | str join ' ')"
        } else {
            ^$env.CNTRCTL ...$args
        }
    }

    export def up [--reset] {
        if $reset {
            const d = path self data/postgres/data/
            print $"rm -rf ($d)"
            sudo rm -rf $d
        }
        let cfg = open $CHAT | get database
        dcr chat_db
        start
        wait-cmd -t 'wait postgresql' {
            ^$env.CNTRCTL ...[
                exec chat_db
                bash -c
                $'pg_isready -U ($cfg.user)'
            ]
        }
    }

    export def migrate [] {
        cargo run --bin migrate
    }
}

module rpk  {
    export def send [
        data
        --partition:int=0
        --topic(-t):string@"topic list"
        --patch: any
    ] {
        let c = open $CFG
        let dp = match ($data | describe -d).type {
            record => {{}},
            list => [],
            _ => {}
        }
        let record =  $data
        | merge deep ($patch | default $dp)
        | wrap value
        | insert partition $partition
        let data = { records: $record } | to json -r
        http post -H [
            Content-Type application/vnd.kafka.json.v2+json
        ] $"http://($c.redpanda.admin)/topics/($topic)" $data
    }

    export def subscribe [topic:string@"topic list"] {
        let c = open $CFG
        let data = { topics: [$topic] } | to json -r
        curl -sL $"http://($c.redpanda.admin)/topics/($topic)/partitions/0/records?offset=0" -H "Content-Type: application/vnd.kafka.json.v2+json" --data $data
    }

    export def consume [topic:string@"topic list"] {
        mut args = [exec -it redpanda]
        $args ++= [rpk topic consume $topic]
        ^$env.CNTRCTL ...$args
    }

    export def 'group list' [] {
        mut args = [exec -it redpanda]
        $args ++= [rpk group list]
        ^$env.CNTRCTL ...$args | from ssv
    }

    export def 'group delete' [group:string@"group list"] {
        mut args = [exec -it redpanda]
        $args ++= [rpk group delete $group]
        ^$env.CNTRCTL ...$args
    }

    export def 'topic list' [] {
        let c = open $CFG
        http get $"http://($c.redpanda.admin)/topics" | from json
    }

    export def 'topic create' [name:string] {
        mut args = [exec -t redpanda]
        $args ++= [rpk topic create $name]
        ^$env.CNTRCTL ...$args
    }

    export def 'topic delete' [name:string@'topic list'] {
        mut args = [exec -t redpanda]
        $args ++= [rpk topic delete $name]
        ^$env.CNTRCTL ...$args
    }

    export def start [
        --dry-run
        --external: string@cmpl-external = 'localhost'
    ] {
        let image = 'docker.io/redpandadata/redpanda:latest'
        mut args = [run -d --name redpanda]
        let ports = {
            '18081': 18081
            '18082': 18082
            '19092': 19092
            '19644': 9644
        }
        for i in ($ports | transpose k v) {
            $args ++= [-p $"($i.k):($i.v)"]
        }
        let envs = {
        }
        for i in ($envs | transpose k v) {
            $args ++= [-e $"($i.k)=($i.v)"]
        }
        $args ++= [$image]
        $args ++= [
            redpanda
            start
            --kafka-addr
            'internal://0.0.0.0:9092,external://0.0.0.0:19092'
            --advertise-kafka-addr
            $'internal://127.0.0.1:9092,external://($external):19092'
            --pandaproxy-addr
            'internal://0.0.0.0:8082,external://0.0.0.0:18082'
            --advertise-pandaproxy-addr
            $'internal://127.0.0.1:8082,external://($external):18082'
            --schema-registry-addr
            'internal://0.0.0.0:8081,external://0.0.0.0:18081'
            --rpc-addr
            localhost:33145
            --advertise-rpc-addr
            localhost:33145
            --mode
            dev-container
            --smp 1
            --default-log-level=info
        ]
        if $dry_run {
            print $"($env.CNTRCTL) ($args | str join ' ')"
        } else {
            ^$env.CNTRCTL ...$args
        }
    }

    export def up [
        --product
        --consume
        --external: string@cmpl-external = 'localhost'
    ] {
        dcr redpanda
        start --external $external

        wait-cmd -t 'wait redpanda' {
            ^$env.CNTRCTL ...[
                exec redpanda
                rpk cluster info
            ]
        }

        let s = open $GW
        topic create $s.queue.outgo.topic
        topic create $s.queue.income.topic.0

        if $product {
            send --topic $s.queue.outgo.topic (open data/message/event.yaml)
        }

        if $consume {
            consume $s.queue.outgo.topic
        }
    }
}

module iggy {
    export def up [
        --dry-run
    ] {
        let image = 'apache/iggy:latest'
        let name = 'iggy'
        mut args = [run -d --name $name]
        for i in [3000 8080 8090 8092] {
            let pi = $"1($i)" | into int
            let rp = port $pi
            if $rp != $pi {
                print $"(ansi grey)Port ($i) is already in use, switching to ($rp)(ansi reset)"
            }
            $args ++= [-p $"($rp):($i)"]
        }
        let envs = {
            IGGY_ROOT_USERNAME: 'iggy'
            IGGY_ROOT_PASSWORD: 'iggy'
        }
        for i in ($envs | transpose k v) {
            $args ++= [-e $"($i.k)=($i.v)"]
        }
        let data = [$WORKDIR data] | path join
        $args ++= [-v $"($data):/local_data"]
        $args ++= [
            --cap-add SYS_NICE
            --security-opt seccomp=unconfined
            --ulimit memlock=-1:-1
        ]
        $args ++= [$image]

        if $dry_run {
            print $"($env.CNTRCTL) ($args | str join ' ')"
        } else {
            ^$env.CNTRCTL ...$args
            let args = [exec -it $name iggy --username iggy --password iggy]
            ^$env.CNTRCTL ...[...$args stream create fluxora]
            ^$env.CNTRCTL ...[...$args topic create fluxora event 1 none]
            ^$env.CNTRCTL ...[...$args topic create fluxora push 1 none]
        }

    }
}

module ui {
    export def up [] {
        let t = open $CFG | get dx
        cd crates/ui
        ^dx serve --port $t.port
    }

    export def build [] {
        cd crates/ui
        rm -rf .../target/dx/ui/release/web/public/
        ^dx build --web --release
        dust .../target/dx/ui/release/web/public/
    }

    export def 'border flashing' [] {
        for _ in 1.. {
            for i in [primary, disable, secondary, accent] {
                sleep 0.2sec
                send 00.chat.layout.yaml -p {
                    data: {
                        sub: [
                            {},
                            {item:
                                [
                                    {},
                                    {attrs: {class: $'box border shadow nogrow s as ($i)'}}
                                ]
                            }
                        ]
                    }
                }
            }
        }
    }



    export def 'message concat' [] {
        for _ in 1.. {
            send 02.concat.yaml
            sleep 0.8sec
        }
    }

    export def 'message replace' [] {
        for _ in 1.. {
            send 02.replace.yaml
            sleep 0.8sec
        }
    }

    export def 'export css' [] {
        use git *
        use git/shortcut.nu *
        use lg
        lg level 1 'begin'
        cp crates/ui/assets/main.css ../ydncf/index.css
        let msg = git-last-commit
        let msg = $"($msg.message)\n\n($msg.body)"
        cd ../ydncf
        if (git-changes | is-not-empty) {
            git add .
            git commit -m $msg
            git push
        }
        lg level 1 'end'
    }
}

module hooks {
    def cmpl-reg [] {
        open $CFG | get hooks | columns
    }

    export def list [] {
        let c = open $CFG | get server
        http get $"http://($c.host)/config/hooks"
    }

    export def upload [name: string@cmpl-reg] {
        let c = open $CFG
        let d = $c | get hooks | get $name
        let h = $c.server.host
        for i in ($d | transpose k v) {
            http post --allow-errors --content-type application/json $"http://($h)/config/hooks/($i.k)" $i.v
        }
    }
}

module chat {
    use pg
    export def up [
        --pg
    ] {
        if $pg {
            pg up
        }
        cargo run --bin chat
    }

    export def 'container up' [
        --external: string@cmpl-external = 'host.docker.internal'
    ] {
        let image = 'ghcr.io/fj0r/fluxora:chat'
        ^$env.CNTRCTL pull $image
        let config = mktemp -t --suffix chat
        open -r chat.toml
        | str replace -a localhost $external
        | save -f $config
        ^$env.CNTRCTL run ...[
            --name fluxora-chat
            --rm -it
            -p 3003:3003
            -v $"($config):/app/chat.toml"
            -w /app
            $image
        ]
    }


    export def build [] {
        cargo build --release --bin chat
    }
}

module gw {
    use rpk

    export def up [
        --rpk
        --external: string@cmpl-external = 'localhost'
    ] {
        if $rpk {
            rpk up --external $external
        }
        cargo run --bin gateway
        watch gateway --glob **/*.rs -q {|op, path, newPath|
            if $op not-in ['Write'] { return }

            let x = ps -l | where command == target/debug/gateway
            if ($x | is-not-empty) {
                kill $x.pid
            }
            cargo run --bin gateway
        }
    }

    export def 'container up' [
        --external: string@cmpl-external = 'host.docker.internal'
    ] {
        let image = 'ghcr.io/fj0r/fluxora:gateway'
        ^$env.CNTRCTL pull $image
        ^$env.CNTRCTL run ...[
            --name fluxora-gateway
            --rm -it
            -p 3000:3000
            -e $"GATEWAY_QUEUE_OUTGO_BROKER=[($external):19092]"
            -e $"GATEWAY_QUEUE_INCOME_BROKER=[($external):19092]"
            -w /app
            $image
        ]
    }

    export def build [] {
        $env.RUSTFLAGS = "--cfg tokio_unstable"
        cargo build --release --bin gateway
    }

    export def profile [] {
        cargo profiler callgrind --bin target/release/gateway
        kcachegrind callgrind.out
        rm callgrind.out
    }

    export def client [] {
        let c = open $CFG
        websocat $"ws://($c.server.host)/channel"
    }

}

module test { 
    export def serve [] {
        let ji = job spawn { dev serve }
        sleep 2sec
        do -i {
            dev client
        }
        job kill $ji
    }

    export def wsconn [] {
        for i in 0..100 {
            job spawn {
                websocat $"ws://localhost:3000/channel?token=abc($i)"
            }
        }
    }

    export def render [] {
        curl -H 'Content-Type: application/json' -X POST http://localhost:3000/debug/render/user.json -d'{"info": {"username": "test"}}'
    }

    export def benchmark [n: int] {
        #drill -b drill.yaml -s
        let url = [
            http://localhost:3000/admin/sessions
            http://localhost:3003/v1/user/alice
            http://localhost:3003/v1/users
        ]
        let url = $url | get $n
        print $"====> ($url)"
        oha -c 50 -n 200000 $url
    }
}

export use test
export use iggy
export use rpk
export use gw
export use chat
export use pg
export use hooks
export use ui

export def receiver [] {
    let c = open $CFG
    http get $"http://($c.server.host)/admin/sessions"
}

def cmpl-act [] {
    [Message Layout test]
}

def cmpl-data [] {
    cd ([$WORKDIR data message] | path join)
    ls | get name
}

def cmpl-external [] {
    ip route
    | lines
    | parse -r ([
        '(?<default>default via)?'
        '(?<gateway>[0-9\./]+)'
        'dev (?<dev>[\w\-]+)'
        'proto (?<proto>dhcp|kernel scope link)'
        'src (?<src>[0-9\.]+)'
    ] | str join '\s*')
    | get src
    | uniq
    | prepend ['host.docker.internal' 'host.containers.internal']
    | { completions: $in, options: { sort: false } }
}

export def send [
    file:string@cmpl-data
    --receiver(-r): list<string>@receiver = []
    --sender(-s): string = 'unknown'
    --patch(-p): any
    --full
    --rpk
    --topic(-t):string@"rpk topic list" = "push"
    --partition:int=0
] {
    let f = if $full { $file } else {
        [$WORKDIR data message $file] | path join
    }
    let content = open $f
    let dp = match ($content | describe -d).type {
        record => {{}},
        list => [],
        _ => {}
    }
    let data = {
        receiver: $receiver,
        sender: $sender,
        content: ($content | merge deep ($patch | default $dp))
    }
    let c = open $CFG
    if $rpk {
        let data = {
            records: {
                value: $data
                partition: $partition
            }
        }
        | to json -r
        http post -H [
            Content-Type application/vnd.kafka.json.v2+json
        ] $"http://($c.redpanda.admin)/topics/($topic)" $data
    } else {
        let c = $c | get server
        let host = $"http://($c.host)/admin/send"
        http post --content-type application/json $host $data
    }
}

export def 'watch message' [] {
    watch data/message {|op, path|
        if $op not-in ['Write'] { return }
        send --full $path
    }
}

export def 'serve' [
    --rpk
    --external: string@cmpl-external = 'localhost'
] {
    if $rpk {
        rpk up --external $external
    }
    #$env.RUST_BACKTRACE = 1
    #$env.GATEWAY_KAFKA_ENABLE = 1
    let g = job spawn {
        gw up
    }
    ui up
    job kill $g
}

export def 'update images' [] {
    let images = ['fj0r/fluxora:gateway', 'fj0r/fluxora:chat' ]
    for i in $images {
        dpl $"ghcr.lizzie.fun/($i)" --rename $"ghcr.io/($i)"
    }
}

export def clippy [dir] {
    cd $dir
    cargo clippy
}

export def jsonschema [] {
    cargo run --example brickschema --features=schema
}

export def brick_test [] {
    cargo run --example scratch --features="scratch"
}

export def gen-type [] {
    jsonschema
    | datamodel-codegen --output layout.py --input-file-type jsonschema --target-python-version 3.12
}

export def git-hooks [act ctx] {
    if $act == 'pre-commit' and $ctx.branch == 'main' {
        cargo fmt
        git add .
    }
}

module macro {
    export def brick [] {
        cargo test -p brick_macro test_macro
    }

    export def ui [] {
        cargo test -p ui_macro
    }
}

export use macro
