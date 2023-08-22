
# Remind

A command line tool to set reminders.

Reminders will create a pop-up in MacOs with a given message after a given delay.
Reminders are non-persistent after a reboot.

## Install:
This will install the `remind` command so that it's available in your terminal.
```
cargo install --force --path .
```

## Example usages:
```
$ remind me to do something in 5 minutes
$ remind me in 1 hour that I have a meeting
$ remind cycling in 30 minutes
$ remind me to stop working at 17:30
```

## Grammar
```
command = 'remind' 'me'? (action | action time | time action)
action = 'to'? message | 'that'? message
message = string
time = 'in' number time_unit | 'at' hour | 'at' minute
time_unit = 'second' | 'seconds' | 'minute' | 'minutes' | 'hour' | 'hours'
hour = number ':' number
minute = number ':' number ':' number
```

### Meta-grammar
The grammar above is written according to the following custom meta-grammar:

- literals: strings within single quotes, e.g. 'some literal'
- rule names: non-quoted words that don't contain spaces, e.g. rule1
- concatenation: rules or literals separated by spaces, e.g. rule1 rule2
- alternatives: rules or literals separated by a vertical line, e.g. rule1 | rule2
- rule 'string': a sequence of characters, e.g. some string
- rule 'number': a sequence of digits, e.g. 123

## Semantic rules

- if no time is specified, 0 seconds is used.

## How the pop-up works

The pop-up is created using AppleScript, which is executed using the `osascript` command.
Something like `osascript -e 'display alert "Hello World"'` will create a pop-up with the message 'Hello World'.

I could have used `display notification`, but the notification only appears for some seconds, which might be easily missed.
