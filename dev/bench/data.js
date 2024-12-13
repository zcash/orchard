window.BENCHMARK_DATA = {
  "lastUpdate": 1734088009424,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira-Emma Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1c426674a855dbc25b088ed9579d52706eea4244",
          "message": "Merge pull request #443 from zcash/pczt-verifier\n\nAdd methods for validating aspects of PCZT bundles",
          "timestamp": "2024-12-13T10:52:58Z",
          "tree_id": "aed6bf9eadd224a93e94f09b12faf7b857d8ce40",
          "url": "https://github.com/zcash/orchard/commit/1c426674a855dbc25b088ed9579d52706eea4244"
        },
        "date": 1734088008729,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2907696417,
            "range": "± 35258300",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2903701389,
            "range": "± 10618988",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4178065272,
            "range": "± 25312782",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5425843484,
            "range": "± 9566159",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25429950,
            "range": "± 507688",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25264809,
            "range": "± 574880",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28389194,
            "range": "± 648513",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 32104802,
            "range": "± 571374",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1516613,
            "range": "± 9157",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 126355,
            "range": "± 599",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1515031,
            "range": "± 9108",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1343208616,
            "range": "± 4083817",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16031698,
            "range": "± 17282",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2148953,
            "range": "± 4715",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15994618,
            "range": "± 20498",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2124312,
            "range": "± 7725",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80109921,
            "range": "± 256106",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10695972,
            "range": "± 42515",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 79948555,
            "range": "± 819350",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10570979,
            "range": "± 32403",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 160280746,
            "range": "± 2235530",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21392394,
            "range": "± 587249",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 159851521,
            "range": "± 510231",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21123653,
            "range": "± 48405",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465843,
            "range": "± 2871",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 501011,
            "range": "± 970",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}