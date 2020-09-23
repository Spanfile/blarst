So anyway, I started blasting.

![so anyway, I started blasting](https://i.kym-cdn.com/photos/images/newsfeed/001/601/217/ff1.jpg)

# blarst

The stupidest name for a tool. It sends DNS queries via UDP to a given address as fast as it can, and counts how many queries it sends and how many responses it gets back. If "as fast as humanely possible" is too much, the rate can be limited to something slower. It is also possible to only send out a certain amount of queries.

### what is that name tho

The tool is essentially a recreation of [dnsblast](https://github.com/jedisct1/dnsblast). Just like it, this tool blasts queries to a target. `blast` with `rs` slapped in the middle to get `blarst`. It can also be turned into `blart` which is just nice.

## Okay cool, how do I use it?

`blarst <address>` will spit out as many queries as possible, and display:
- how many queries it's sent out, how many responses it's got back
- the ratio between the two scalars
- how quick the queries are going out in queries per second
- how quick the responses are coming in in responses per second

Like any self-respecting CLI tool, it has some options available:
- `-r`, `--rate`: the target queries per second to try and send out. A value of 0 means sending as fast as possible, which corresponds to trying to send a query once every nanosecond. It also means you can't try sending more than a query per nanosecond.
- `-p`, `--port`: target a certain port instead of the default 53
- `-u`, `--update`: update the status display N times every second
- `-c`, `--count`: send only this many queries and then stop
