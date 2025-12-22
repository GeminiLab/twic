# Twic

```plaintext
msg:hello!,from:twic;
```

Twic (Tiny Writable Inline Config) is an experimental lightweight data exchange format focusing on extremely minimalistic but writable syntax for small, simple and inline configurations and data snippets, while remaining fully expressive for complex data structures when needed.

Here’s what a simple Twic snippet looks like, and its equivalent JSON for comparison:

Twic:

```plaintext
profile:name:twic,version:0.1;,users::alice,bob;;
```

JSON:

```json
{
  "profile": {
    "name": "twic",
    "version": 0.1
  },
  "users": ["alice", "bob"]
}
```

## Usage

### Working with untyped data

Use [`twic::value`](`value`) module to manipulate untyped Twic data, with recursive enum representation similar to JSON. The enum type `twic::Value` represents any valid Twic value, and is defined as follows:

```rust,ignore
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Vector(Vec<Value>),
    Map(Map<String, Value>),
}
```

## Syntax

Twic supports 6 data types (same as JSON): null, boolean, number, string, vector (array) and map (object). Here are the syntax rules for each data type:

### Null and Boolean

- Case-sensitive keywords `null`, `true` and `false` are used to represent null and boolean values.
- Alternative representations found in some other formats (like `nil`, `yes`, `no`, `1`, `0`) are not supported.

### Number

- Integers and floating-point numbers are supported.
- NaNs (`nan`) and infinities (`inf`, `+inf`, `-inf`) are supported with case-sensitive keywords.
- Signs `+` and `-` can be used for positive and negative numbers respectively.
- Leading zeros are allowed in decimal integers.
- Hexadecimal integers are supported with a case-sensitive `0x` prefix. Leading zeros are allowed in hexadecimal integers. Uppercase and lowercase `A-F`s are both allowed.
- Leading or trailing decimal points are not allowed in floating-point numbers.

### String

- Strings can be unquoted or quoted. Quoted strings are enclosed in double quotes (`"`).
- Escape sequences are supported only in quoted strings. JSON escape sequences (namely `\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t`) are supported. Unicode escape sequences `\uXXXX` (with exactly 4 hexadecimal digits) and `\u{X...}` (with 1 to 8 hexadecimal digits) are also supported. Byte sequences can be represented using `\xXX` (with exactly 2 hexadecimal digits).
- Unquoted strings are non-empty sequences of non-whitespace characters that
  - do not contain any of `:`, `;`, `,`,
  - do not start with any of `"` (double quote), `0-9` (digit), `+`, `-`, and
  - are not equal to any of the keywords: `null`, `true`, `false`, `nan`, `inf`.

### Vector

- Vectors start with `:` and end with `;`.
- Elements are separated by `,` (comma).

### Map

- Maps end with `;`.
- Key-value pairs are separated by `,` (comma).
- Keys and values are separated by `:` (colon).

### Whitespaces and Special Characters

- Twic uses a minimal set of special characters: `:`, `;`, `,` for structural purposes, and `"` for quoted strings, any other non-whitespace character outside quoted strings is considered part of the data,
  - `:` is used to denote the start of a vector or to separate keys and values in a map,
  - `;` is used to denote the end of a vector or map,
  - `,` is used to separate elements in a vector or key-value pairs in a map.
- Whitespaces (any combination of unicode whitespace characters) can appear between any two values, or between structural characters and values, and are ignored.

## Formal Specification

```plaintext
value      = null | boolean | number | string | vector | map
null       = "null"
boolean    = "true" | "false"
number     = decimal | hex | special
decimal    = [ "+" | "-" ] int [ frac ] [ exp ]
hex        = [ "+" | "-" ] "0x" hex_digit { hex_digit }
special    = "nan" | [ "+" | "-" ] "inf"
int        = digit { digit }
frac       = "." digit { digit }
exp        = ( "e" | "E" ) [ "+" | "-" ] digit { digit }
hex_digit  = digit | "a" | "b" | "c" | "d" | "e" | "f" | "A" | "B" | "C" | "D" | "E" | "F"
digit      = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
string     = unquoted_string | quoted_string
unquoted_string = (? any identifier-like string that does not conflict with other types ?)
quoted_string   = (? double-quoted string with escape sequences ?)
vector     = ":" [ value { "," value } ] ";"
map        = [ key_value { "," key_value } ] ";"
key_value  = string ":" value
```
