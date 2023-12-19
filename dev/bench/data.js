window.BENCHMARK_DATA = {
  "lastUpdate": 1703017029309,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "str4d",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "78f598616a07749e7071fd30bc63bf5f39372257",
          "message": "Merge pull request #404 from nuttycom/builder_functions\n\nAdd a public bundle-builder function as an alternative to the mutable builder.",
          "timestamp": "2023-12-19T20:03:08Z",
          "tree_id": "f0e0208f1b6495f8ab30745cfd0c6cd9c8c03238",
          "url": "https://github.com/zcash/orchard/commit/78f598616a07749e7071fd30bc63bf5f39372257"
        },
        "date": 1703017027699,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2911247675,
            "range": "± 32341376",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2896637842,
            "range": "± 6164064",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4133591862,
            "range": "± 16572824",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5433009823,
            "range": "± 20455299",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24961930,
            "range": "± 543110",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25093772,
            "range": "± 497680",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28074324,
            "range": "± 482707",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31673162,
            "range": "± 534933",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1528248,
            "range": "± 7092",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127641,
            "range": "± 645",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1524510,
            "range": "± 3033",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1337717119,
            "range": "± 2444757",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16119206,
            "range": "± 42071",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2172558,
            "range": "± 13222",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16107319,
            "range": "± 27248",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2124265,
            "range": "± 8251",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80542641,
            "range": "± 241680",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10809348,
            "range": "± 63592",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80457613,
            "range": "± 611144",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10572754,
            "range": "± 44457",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161025172,
            "range": "± 230131",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21588897,
            "range": "± 68433",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160934684,
            "range": "± 3296557",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21136901,
            "range": "± 69240",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466747,
            "range": "± 2014",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 502070,
            "range": "± 1164",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}