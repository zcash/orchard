window.BENCHMARK_DATA = {
  "lastUpdate": 1651878454201,
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
          "id": "b1a5c1a635b73c5c9196d3a27ce7d56951701399",
          "message": "Merge pull request #326 from zcash/fix-note-decryption-benchmark\n\nMake note decryption benchmark reliable",
          "timestamp": "2022-05-06T23:50:43+01:00",
          "tree_id": "306401f79a4aa06322a7da39f568c64a5162223f",
          "url": "https://github.com/zcash/orchard/commit/b1a5c1a635b73c5c9196d3a27ce7d56951701399"
        },
        "date": 1651878453171,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4048895706,
            "range": "± 30412216",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4046109649,
            "range": "± 13228237",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5791424639,
            "range": "± 13933628",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7525867176,
            "range": "± 9906447",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 33005621,
            "range": "± 335435",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 33094697,
            "range": "± 330715",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 36934139,
            "range": "± 352234",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 40405376,
            "range": "± 667157",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1045140,
            "range": "± 788",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 132470,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1041958,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 134665661,
            "range": "± 47294",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 20564891,
            "range": "± 8163",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2338955,
            "range": "± 598",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 20515818,
            "range": "± 8443",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2303275,
            "range": "± 1288",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 102747032,
            "range": "± 50546",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 11637650,
            "range": "± 3439",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 102538084,
            "range": "± 32926",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11458191,
            "range": "± 4692",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 205520681,
            "range": "± 408478",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 23266467,
            "range": "± 7907",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 205113319,
            "range": "± 70891",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 22917278,
            "range": "± 48780",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 493421,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 546815,
            "range": "± 311",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}