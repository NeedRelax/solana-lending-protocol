/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/mock_writer.json`.
 */
export type MockWriter = {
  "address": "BN6SmdB35pe6CF8dA7AehZMB3umBGPQK46TcyB9PNBLc",
  "metadata": {
    "name": "mockWriter",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "createFakePyth",
      "discriminator": [
        152,
        208,
        108,
        132,
        222,
        75,
        138,
        27
      ],
      "accounts": [
        {
          "name": "fakePythAccount",
          "writable": true
        }
      ],
      "args": [
        {
          "name": "price",
          "type": "i64"
        },
        {
          "name": "conf",
          "type": "u64"
        },
        {
          "name": "expo",
          "type": "i32"
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "accountDataTooSmall",
      "msg": "The provided account data is too small."
    }
  ]
};
