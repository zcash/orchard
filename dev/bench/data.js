window.BENCHMARK_DATA = {
  "lastUpdate": 1703031474680,
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
          "id": "c3756967875185e37958a8b2f321026add9a894a",
          "message": "Merge pull request #405 from daira/change-license\n\nChange license from BOSL to \"MIT OR Apache-2.0\"",
          "timestamp": "2023-12-20T00:03:42Z",
          "tree_id": "3c5cee0dade42a81928d1e9c713e72e3e8da4826",
          "url": "https://github.com/zcash/orchard/commit/c3756967875185e37958a8b2f321026add9a894a"
        },
        "date": 1703031473056,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2895044143,
            "range": "± 19397358",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2898305633,
            "range": "± 20469010",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4146595166,
            "range": "± 13881588",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5396915204,
            "range": "± 32923875",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25332566,
            "range": "± 622739",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25402472,
            "range": "± 549390",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28113037,
            "range": "± 533794",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31578620,
            "range": "± 405414",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1525682,
            "range": "± 4697",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127603,
            "range": "± 426",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1522167,
            "range": "± 4801",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1340201195,
            "range": "± 3053960",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16090607,
            "range": "± 18596",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2171484,
            "range": "± 2086",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16077685,
            "range": "± 28983",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2125836,
            "range": "± 7507",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80405376,
            "range": "± 226460",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10805946,
            "range": "± 44490",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80331306,
            "range": "± 350029",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10575575,
            "range": "± 32485",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 160732544,
            "range": "± 318615",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21590929,
            "range": "± 744398",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160685896,
            "range": "± 388613",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21127151,
            "range": "± 595635",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 466940,
            "range": "± 2149",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 502402,
            "range": "± 1914",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}