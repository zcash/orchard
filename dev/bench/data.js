window.BENCHMARK_DATA = {
  "lastUpdate": 1647556771155,
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
          "id": "0ee6cf894ff73a83e97d4f7b5cf0066083293b2c",
          "message": "Merge pull request #296 from zcash/commitivk-identity-error\n\nspec.rs: Check that commit_ivk returns a nonzero base.",
          "timestamp": "2022-03-17T22:27:27Z",
          "tree_id": "f586c99e272ff2ab2c32cfc455079f1e9955c57c",
          "url": "https://github.com/zcash/orchard/commit/0ee6cf894ff73a83e97d4f7b5cf0066083293b2c"
        },
        "date": 1647556770171,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4120502896,
            "range": "± 18766219",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4121461240,
            "range": "± 13740593",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5904200147,
            "range": "± 19139222",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7689166046,
            "range": "± 32719692",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 31983784,
            "range": "± 297157",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 32154860,
            "range": "± 326003",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 36261834,
            "range": "± 255613",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 40032627,
            "range": "± 278646",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}