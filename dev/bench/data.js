window.BENCHMARK_DATA = {
  "lastUpdate": 1703172645992,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "kris@nutty.land",
            "name": "Kris Nuttycombe",
            "username": "nuttycom"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ba70c32c280aa2dc23e4caa82252adc2f28cf17a",
          "message": "Merge pull request #409 from zcash/required_bundles\n\nModify `BundleType` to exclude the anchor & allow no bundle to be produced.",
          "timestamp": "2023-12-21T08:16:32-07:00",
          "tree_id": "dd57058b3b56ea94648f732c73217e1d62538c23",
          "url": "https://github.com/zcash/orchard/commit/ba70c32c280aa2dc23e4caa82252adc2f28cf17a"
        },
        "date": 1703172644969,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2901333158,
            "range": "± 70830809",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2901416887,
            "range": "± 21449278",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4137787718,
            "range": "± 34796512",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5391394872,
            "range": "± 11610064",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24816169,
            "range": "± 521388",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25341853,
            "range": "± 557203",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28116588,
            "range": "± 549371",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31633102,
            "range": "± 397359",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1527600,
            "range": "± 4858",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127531,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1525143,
            "range": "± 5370",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1351390765,
            "range": "± 2462026",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16121893,
            "range": "± 14430",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2168820,
            "range": "± 6759",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16101778,
            "range": "± 25101",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2139644,
            "range": "± 2119",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80556234,
            "range": "± 900306",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10787598,
            "range": "± 26858",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80471678,
            "range": "± 122168",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10641250,
            "range": "± 26874",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161118687,
            "range": "± 259613",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21573769,
            "range": "± 61561",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160979953,
            "range": "± 2317972",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21274612,
            "range": "± 65046",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466892,
            "range": "± 2516",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 502311,
            "range": "± 2915",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}