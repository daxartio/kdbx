# Test fixtures

All credentials in these fixtures are public, test-only dummy values.

## `test.kdbx`

KDBX 4.0 fixture used to test reading entries and upgrading to KDBX 4.1 when saving. Dummy master password: `test123`, with key file `secret`.

## `kdbx3_1.kdbx`

- Source: [keepass-rs v0.17.0, `test_db_with_password.kdbx`](https://github.com/sseemayer/keepass-rs/blob/v0.17.0/tests/resources/test_db_with_password.kdbx).
- Upstream license: [MIT](https://github.com/sseemayer/keepass-rs/blob/v0.17.0/LICENSE).
- Git blob: `74f0cdca430fecf1538c62bf6920a996b90eab7a`.
- Dummy master password: `demopass`; no key file.
- Purpose: verify that saving unsupported KDBX 3.1 files fails without modifying the original. A binary fixture is necessary because keepass 0.17 can only generate KDBX 4.1 files.

### Upstream license notice

MIT License

Copyright (c) 2019 Stefan Seemayer

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
