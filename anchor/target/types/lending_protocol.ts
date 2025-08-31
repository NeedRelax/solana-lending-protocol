/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/lending_protocol.json`.
 */
export type LendingProtocol = {
  "address": "2XsQQ3t5uScXfiwxWGBLNXBSMwoMfEyw9Muc1LwcC7gH",
  "metadata": {
    "name": "lendingProtocol",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "addAssetPool",
      "docs": [
        "[Governance] Adds a new asset pool to the market."
      ],
      "discriminator": [
        219,
        121,
        194,
        180,
        186,
        22,
        121,
        206
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "writable": true
        },
        {
          "name": "assetPool",
          "writable": true,
          "signer": true
        },
        {
          "name": "assetVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  115,
                  115,
                  101,
                  116,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              }
            ]
          }
        },
        {
          "name": "assetMint"
        },
        {
          "name": "governanceAuthority",
          "writable": true,
          "signer": true,
          "relations": [
            "marketConfig"
          ]
        },
        {
          "name": "pythPriceFeedAccount"
        },
        {
          "name": "chainlinkPriceFeedAccount",
          "optional": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "assetPoolParams"
            }
          }
        }
      ]
    },
    {
      "name": "approveDelegation",
      "docs": [
        "[User] Approves another account to borrow against their position."
      ],
      "discriminator": [
        101,
        244,
        227,
        116,
        198,
        137,
        117,
        56
      ],
      "accounts": [
        {
          "name": "creditDelegation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  95,
                  100,
                  101,
                  108,
                  101,
                  103,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "owner"
              },
              {
                "kind": "account",
                "path": "assetPool"
              },
              {
                "kind": "account",
                "path": "delegateeAccount"
              }
            ]
          }
        },
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "delegateeAccount"
        },
        {
          "name": "assetPool"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "borrow",
      "docs": [
        "[User] Borrows assets against their deposited collateral."
      ],
      "discriminator": [
        228,
        253,
        131,
        202,
        207,
        116,
        89,
        18
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "assetPool",
          "writable": true
        },
        {
          "name": "userPosition",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "user",
          "signer": true
        },
        {
          "name": "userAssetAccount",
          "writable": true
        },
        {
          "name": "assetVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  115,
                  115,
                  101,
                  116,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              }
            ]
          }
        },
        {
          "name": "assetMint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "pythPriceFeedAccount"
        },
        {
          "name": "chainlinkPriceFeedAccount"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "borrowDelegated",
      "docs": [
        "[Delegatee] Borrows using the credit line delegated to them."
      ],
      "discriminator": [
        102,
        111,
        123,
        56,
        76,
        129,
        207,
        85
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "assetPool",
          "writable": true
        },
        {
          "name": "assetVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  115,
                  115,
                  101,
                  116,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              }
            ]
          }
        },
        {
          "name": "assetMint"
        },
        {
          "name": "ownerPosition",
          "writable": true
        },
        {
          "name": "owner"
        },
        {
          "name": "creditDelegation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  95,
                  100,
                  101,
                  108,
                  101,
                  103,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "owner"
              },
              {
                "kind": "account",
                "path": "assetPool"
              },
              {
                "kind": "account",
                "path": "delegatee"
              }
            ]
          }
        },
        {
          "name": "delegatee",
          "signer": true
        },
        {
          "name": "delegateeTokenAccount",
          "writable": true
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "pythPriceFeedAccount"
        },
        {
          "name": "chainlinkPriceFeedAccount"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "collectProtocolFees",
      "docs": [
        "[Governance] Collects accrued protocol fees from an asset pool."
      ],
      "discriminator": [
        22,
        67,
        23,
        98,
        150,
        178,
        70,
        220
      ],
      "accounts": [
        {
          "name": "marketConfig"
        },
        {
          "name": "assetPool",
          "writable": true
        },
        {
          "name": "assetVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  115,
                  115,
                  101,
                  116,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              }
            ]
          }
        },
        {
          "name": "destinationAccount",
          "writable": true
        },
        {
          "name": "assetMint"
        },
        {
          "name": "governanceAuthority",
          "signer": true,
          "relations": [
            "marketConfig"
          ]
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": []
    },
    {
      "name": "createUserPosition",
      "docs": [
        "[User] Initializes a user's position account for a specific asset pool."
      ],
      "discriminator": [
        6,
        137,
        127,
        227,
        135,
        241,
        14,
        109
      ],
      "accounts": [
        {
          "name": "userPosition",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "assetPool"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "deposit",
      "docs": [
        "[User] Deposits assets into a pool to be used as collateral."
      ],
      "discriminator": [
        242,
        35,
        198,
        137,
        82,
        225,
        242,
        182
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "assetPool",
          "writable": true
        },
        {
          "name": "userPosition",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "userAssetAccount",
          "writable": true
        },
        {
          "name": "assetVault",
          "writable": true
        },
        {
          "name": "assetMint"
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "enableWithdrawOnlyMode",
      "docs": [
        "[Governance] Sets the protocol to withdraw-only mode."
      ],
      "discriminator": [
        242,
        168,
        252,
        195,
        96,
        63,
        80,
        164
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "governanceAuthority",
          "signer": true,
          "relations": [
            "marketConfig"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "executeOperations",
      "docs": [
        "[User] Executes a batch of operations in a single transaction."
      ],
      "discriminator": [
        99,
        191,
        213,
        196,
        122,
        40,
        15,
        249
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "assetPool",
          "writable": true
        },
        {
          "name": "assetVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  115,
                  115,
                  101,
                  116,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              }
            ]
          }
        },
        {
          "name": "assetMint"
        },
        {
          "name": "userPosition",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "user",
          "signer": true
        },
        {
          "name": "userAssetAccount",
          "writable": true
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "pythPriceFeedAccount"
        },
        {
          "name": "chainlinkPriceFeedAccount"
        }
      ],
      "args": [
        {
          "name": "operations",
          "type": {
            "vec": {
              "defined": {
                "name": "operation"
              }
            }
          }
        }
      ]
    },
    {
      "name": "flashLoan",
      "docs": [
        "[User/Bot] Executes a flash loan."
      ],
      "discriminator": [
        239,
        246,
        59,
        224,
        139,
        20,
        175,
        14
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "assetPool",
          "writable": true
        },
        {
          "name": "assetVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  115,
                  115,
                  101,
                  116,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              }
            ]
          }
        },
        {
          "name": "destinationAccount",
          "writable": true
        },
        {
          "name": "assetMint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "flashLoanReceiverProgram"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "callbackIxData",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "initializeMarketConfig",
      "docs": [
        "[Governance] Initializes the central market configuration."
      ],
      "discriminator": [
        5,
        94,
        211,
        203,
        75,
        239,
        159,
        255
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "liquidate",
      "docs": [
        "[Liquidator] Liquidates an unhealthy position by repaying debt to seize collateral."
      ],
      "discriminator": [
        223,
        179,
        226,
        125,
        48,
        46,
        39,
        74
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "collateralPool",
          "writable": true
        },
        {
          "name": "loanPool",
          "writable": true
        },
        {
          "name": "collateralMint"
        },
        {
          "name": "loanMint"
        },
        {
          "name": "borrowerCollateralPosition",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "collateralPool"
              },
              {
                "kind": "account",
                "path": "borrower"
              }
            ]
          }
        },
        {
          "name": "borrowerLoanPosition",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "loanPool"
              },
              {
                "kind": "account",
                "path": "borrower"
              }
            ]
          }
        },
        {
          "name": "borrower"
        },
        {
          "name": "liquidator",
          "signer": true
        },
        {
          "name": "liquidatorCollateralAccount",
          "writable": true
        },
        {
          "name": "liquidatorLoanAccount",
          "writable": true
        },
        {
          "name": "collateralVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  115,
                  115,
                  101,
                  116,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "collateralPool"
              }
            ]
          }
        },
        {
          "name": "loanVault",
          "writable": true
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "collateralPriceFeedAccount"
        },
        {
          "name": "collateralChainlinkFeedAccount"
        },
        {
          "name": "loanPriceFeedAccount"
        },
        {
          "name": "loanChainlinkFeedAccount"
        }
      ],
      "args": [
        {
          "name": "amountToRepay",
          "type": "u64"
        }
      ]
    },
    {
      "name": "pauseProtocol",
      "docs": [
        "[Governance] Pauses the protocol, halting most user interactions."
      ],
      "discriminator": [
        144,
        95,
        0,
        107,
        119,
        39,
        248,
        141
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "governanceAuthority",
          "signer": true,
          "relations": [
            "marketConfig"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "repay",
      "docs": [
        "[User] Repays a loan to the asset pool."
      ],
      "discriminator": [
        234,
        103,
        67,
        82,
        208,
        234,
        219,
        166
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "assetPool",
          "writable": true
        },
        {
          "name": "userPosition",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "user",
          "signer": true
        },
        {
          "name": "userAssetAccount",
          "writable": true
        },
        {
          "name": "assetVault",
          "writable": true
        },
        {
          "name": "assetMint"
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "revokeDelegation",
      "docs": [
        "[User] Revokes an existing credit delegation."
      ],
      "discriminator": [
        188,
        92,
        135,
        67,
        160,
        181,
        54,
        62
      ],
      "accounts": [
        {
          "name": "creditDelegation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  105,
                  116,
                  95,
                  100,
                  101,
                  108,
                  101,
                  103,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "owner"
              },
              {
                "kind": "account",
                "path": "assetPool"
              },
              {
                "kind": "account",
                "path": "delegateeAccount"
              }
            ]
          }
        },
        {
          "name": "owner",
          "writable": true,
          "signer": true,
          "relations": [
            "creditDelegation"
          ]
        },
        {
          "name": "delegateeAccount"
        },
        {
          "name": "assetPool"
        }
      ],
      "args": []
    },
    {
      "name": "unpauseProtocol",
      "docs": [
        "[Governance] Unpauses the protocol, resuming normal operations."
      ],
      "discriminator": [
        183,
        154,
        5,
        183,
        105,
        76,
        87,
        18
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "governanceAuthority",
          "signer": true,
          "relations": [
            "marketConfig"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "updateAssetPool",
      "docs": [
        "[Governance] Updates the parameters for an existing asset pool."
      ],
      "discriminator": [
        171,
        90,
        126,
        53,
        250,
        94,
        243,
        130
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "writable": true
        },
        {
          "name": "assetPool",
          "writable": true
        },
        {
          "name": "governanceAuthority",
          "signer": true,
          "relations": [
            "marketConfig"
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "assetPoolParams"
            }
          }
        }
      ]
    },
    {
      "name": "updateGovernanceAuthority",
      "docs": [
        "[Governance] Transfers governance authority to a new public key."
      ],
      "discriminator": [
        11,
        185,
        227,
        55,
        39,
        32,
        168,
        14
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "governanceAuthority",
          "signer": true,
          "relations": [
            "marketConfig"
          ]
        }
      ],
      "args": [
        {
          "name": "newAuthority",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "withdraw",
      "docs": [
        "[User] Withdraws collateral from a pool, subject to health checks."
      ],
      "discriminator": [
        183,
        18,
        70,
        156,
        148,
        109,
        161,
        34
      ],
      "accounts": [
        {
          "name": "marketConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "assetPool",
          "writable": true
        },
        {
          "name": "userPosition",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "user",
          "signer": true
        },
        {
          "name": "userAssetAccount",
          "writable": true
        },
        {
          "name": "assetVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  115,
                  115,
                  101,
                  116,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "assetPool"
              }
            ]
          }
        },
        {
          "name": "assetMint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "pythPriceFeedAccount"
        },
        {
          "name": "chainlinkPriceFeedAccount"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "assetPool",
      "discriminator": [
        81,
        48,
        2,
        215,
        147,
        255,
        152,
        112
      ]
    },
    {
      "name": "creditDelegation",
      "discriminator": [
        192,
        108,
        220,
        35,
        80,
        56,
        106,
        35
      ]
    },
    {
      "name": "marketConfig",
      "discriminator": [
        119,
        255,
        200,
        88,
        252,
        82,
        128,
        24
      ]
    },
    {
      "name": "userPosition",
      "discriminator": [
        251,
        248,
        209,
        245,
        83,
        234,
        17,
        27
      ]
    }
  ],
  "events": [
    {
      "name": "assetPoolAdded",
      "discriminator": [
        139,
        26,
        91,
        19,
        136,
        43,
        84,
        36
      ]
    },
    {
      "name": "assetPoolUpdated",
      "discriminator": [
        89,
        91,
        229,
        75,
        24,
        118,
        18,
        147
      ]
    },
    {
      "name": "borrowed",
      "discriminator": [
        225,
        182,
        241,
        78,
        34,
        145,
        253,
        230
      ]
    },
    {
      "name": "borrowedDelegated",
      "discriminator": [
        237,
        64,
        61,
        124,
        173,
        56,
        5,
        116
      ]
    },
    {
      "name": "delegationUpdated",
      "discriminator": [
        195,
        70,
        246,
        184,
        110,
        77,
        100,
        4
      ]
    },
    {
      "name": "deposited",
      "discriminator": [
        111,
        141,
        26,
        45,
        161,
        35,
        100,
        57
      ]
    },
    {
      "name": "flashLoaned",
      "discriminator": [
        217,
        243,
        240,
        231,
        201,
        56,
        240,
        50
      ]
    },
    {
      "name": "governanceAuthorityChanged",
      "discriminator": [
        92,
        188,
        188,
        99,
        166,
        80,
        99,
        50
      ]
    },
    {
      "name": "liquidation",
      "discriminator": [
        253,
        18,
        85,
        107,
        192,
        175,
        171,
        172
      ]
    },
    {
      "name": "marketConfigInitialized",
      "discriminator": [
        215,
        170,
        151,
        136,
        30,
        87,
        38,
        101
      ]
    },
    {
      "name": "operationsExecuted",
      "discriminator": [
        223,
        153,
        255,
        194,
        38,
        61,
        220,
        152
      ]
    },
    {
      "name": "protocolFeesCollected",
      "discriminator": [
        165,
        34,
        125,
        155,
        15,
        86,
        99,
        191
      ]
    },
    {
      "name": "protocolPaused",
      "discriminator": [
        35,
        111,
        245,
        138,
        237,
        199,
        79,
        223
      ]
    },
    {
      "name": "protocolUnpaused",
      "discriminator": [
        248,
        204,
        112,
        239,
        72,
        67,
        127,
        216
      ]
    },
    {
      "name": "protocolWithdrawOnlyModeEnabled",
      "discriminator": [
        74,
        88,
        180,
        117,
        77,
        77,
        207,
        11
      ]
    },
    {
      "name": "repaid",
      "discriminator": [
        38,
        248,
        231,
        7,
        150,
        164,
        172,
        23
      ]
    },
    {
      "name": "withdrawn",
      "discriminator": [
        20,
        89,
        223,
        198,
        194,
        124,
        219,
        13
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "mathOverflow",
      "msg": "Mathematical overflow during calculation."
    },
    {
      "code": 6001,
      "name": "invalidPythAccount",
      "msg": "Invalid Pyth account provided."
    },
    {
      "code": 6002,
      "name": "pythPriceTooOld",
      "msg": "The Pyth price feed is too old."
    },
    {
      "code": 6003,
      "name": "pythConfidenceTooWide",
      "msg": "The Pyth price confidence interval is too wide."
    },
    {
      "code": 6004,
      "name": "chainlinkPriceTooOld",
      "msg": "The Chainlink price feed is too old."
    },
    {
      "code": 6005,
      "name": "invalidChainlinkPrice",
      "msg": "Invalid Chainlink price (e.g., negative or zero)."
    },
    {
      "code": 6006,
      "name": "allOraclesFailed",
      "msg": "All available oracles failed to provide a valid price."
    },
    {
      "code": 6007,
      "name": "invalidPythPrice",
      "msg": "The Pyth price is invalid (e.g., negative or zero)."
    },
    {
      "code": 6008,
      "name": "insufficientCollateral",
      "msg": "Collateral value is insufficient for this operation."
    },
    {
      "code": 6009,
      "name": "insufficientCollateralAmount",
      "msg": "Not enough collateral deposited to withdraw this amount."
    },
    {
      "code": 6010,
      "name": "positionHealthy",
      "msg": "The position is healthy and cannot be liquidated."
    },
    {
      "code": 6011,
      "name": "positionWouldBecomeUnhealthy",
      "msg": "This operation would leave the position unhealthy."
    },
    {
      "code": 6012,
      "name": "invalidOwner",
      "msg": "The signer is not the owner of the user position account."
    },
    {
      "code": 6013,
      "name": "insufficientCollateralForLiquidation",
      "msg": "Not enough collateral in the position for the liquidation seizure."
    },
    {
      "code": 6014,
      "name": "zeroAmount",
      "msg": "The transaction amount cannot be zero."
    },
    {
      "code": 6015,
      "name": "maxAssetsExceeded",
      "msg": "The maximum number of asset pools has been reached."
    },
    {
      "code": 6016,
      "name": "insufficientLiquidity",
      "msg": "The asset pool has insufficient liquidity for this operation."
    },
    {
      "code": 6017,
      "name": "delegationMismatch",
      "msg": "The signer does not match the approved delegatee for this credit line."
    },
    {
      "code": 6018,
      "name": "delegationExceeded",
      "msg": "The requested borrow amount exceeds the delegated credit line."
    },
    {
      "code": 6019,
      "name": "invalidOperation",
      "msg": "The operation provided in the batch transaction is invalid."
    },
    {
      "code": 6020,
      "name": "delegationIsActive",
      "msg": "The delegation is currently in use and cannot be revoked."
    },
    {
      "code": 6021,
      "name": "flashLoanNotAvailable",
      "msg": "Flash loans are not enabled for this asset pool."
    },
    {
      "code": 6022,
      "name": "flashLoanReentrancy",
      "msg": "Flash loan callback cannot be the lending program itself."
    },
    {
      "code": 6023,
      "name": "flashLoanRepaymentFailed",
      "msg": "The flash loan was not fully repaid with the required fee."
    },
    {
      "code": 6024,
      "name": "collateralMintMismatch",
      "msg": "The provided collateral mint account does not match the one in the collateral pool."
    },
    {
      "code": 6025,
      "name": "loanMintMismatch",
      "msg": "The provided loan mint account does not match the one in the loan pool."
    },
    {
      "code": 6026,
      "name": "protocolPaused",
      "msg": "The protocol is currently paused by governance."
    },
    {
      "code": 6027,
      "name": "protocolNotActive",
      "msg": "The protocol is not active. No new positions or loans can be created."
    },
    {
      "code": 6028,
      "name": "cannotLiquidateSelf",
      "msg": "A liquidator cannot liquidate their own position."
    },
    {
      "code": 6029,
      "name": "invalidOracleAccount",
      "msg": "The provided oracle account is not valid or recognized."
    },
    {
      "code": 6030,
      "name": "invalidLtv",
      "msg": "Loan-to-value cannot be greater than the liquidation threshold."
    },
    {
      "code": 6031,
      "name": "invalidLiquidationThreshold",
      "msg": "Liquidation threshold must be less than 100%."
    },
    {
      "code": 6032,
      "name": "invalidOptimalUtilization",
      "msg": "Optimal utilization must be less than 100%."
    },
    {
      "code": 6033,
      "name": "invalidAssetVault",
      "msg": "The provided asset vault account is invalid for this pool."
    },
    {
      "code": 6034,
      "name": "invalidAssetPool",
      "msg": "The provided asset Pool account is invalid for this pool."
    },
    {
      "code": 6035,
      "name": "invalidAssetMint",
      "msg": "The provided asset Mint account is invalid for this pool."
    }
  ],
  "types": [
    {
      "name": "assetPool",
      "serialization": "bytemuck",
      "repr": {
        "kind": "c",
        "packed": true
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "assetMint",
            "type": "pubkey"
          },
          {
            "name": "assetVault",
            "type": "pubkey"
          },
          {
            "name": "pythPriceFeed",
            "type": "pubkey"
          },
          {
            "name": "chainlinkPriceFeed",
            "type": "pubkey"
          },
          {
            "name": "totalDeposits",
            "type": "u64"
          },
          {
            "name": "totalLoans",
            "type": "u64"
          },
          {
            "name": "accruedProtocolFees",
            "type": "u64"
          },
          {
            "name": "lastInterestUpdateTimestamp",
            "type": "i64"
          },
          {
            "name": "loanToValueBps",
            "type": "u64"
          },
          {
            "name": "liquidationThresholdBps",
            "type": "u64"
          },
          {
            "name": "baseBorrowRateBps",
            "type": "u128"
          },
          {
            "name": "baseSlopeBps",
            "type": "u128"
          },
          {
            "name": "optimalUtilizationBps",
            "type": "u64"
          },
          {
            "name": "kinkSlopeBps",
            "type": "u128"
          },
          {
            "name": "protocolFeeBps",
            "type": "u64"
          },
          {
            "name": "flashLoanFeeBps",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "assetPoolAdded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "poolKey",
            "type": "pubkey"
          },
          {
            "name": "assetMint",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "assetPoolParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "loanToValueBps",
            "type": "u64"
          },
          {
            "name": "liquidationThresholdBps",
            "type": "u64"
          },
          {
            "name": "baseBorrowRateBps",
            "type": "u128"
          },
          {
            "name": "baseSlopeBps",
            "type": "u128"
          },
          {
            "name": "optimalUtilizationBps",
            "type": "u64"
          },
          {
            "name": "kinkSlopeBps",
            "type": "u128"
          },
          {
            "name": "protocolFeeBps",
            "type": "u64"
          },
          {
            "name": "flashLoanFeeBps",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "assetPoolUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "poolKey",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "borrowed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "borrowedDelegated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "delegatee",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "creditDelegation",
      "serialization": "bytemuck",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "delegatee",
            "type": "pubkey"
          },
          {
            "name": "assetPool",
            "type": "pubkey"
          },
          {
            "name": "initialDelegatedAmount",
            "type": "u64"
          },
          {
            "name": "delegatedAmount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "delegationUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "delegatee",
            "type": "pubkey"
          },
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "delegatedAmount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "deposited",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "flashLoaned",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "receiver",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "fee",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "governanceAuthorityChanged",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "oldAuthority",
            "type": "pubkey"
          },
          {
            "name": "newAuthority",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "liquidation",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "collateralPool",
            "type": "pubkey"
          },
          {
            "name": "loanPool",
            "type": "pubkey"
          },
          {
            "name": "liquidator",
            "type": "pubkey"
          },
          {
            "name": "borrower",
            "type": "pubkey"
          },
          {
            "name": "repayAmount",
            "type": "u64"
          },
          {
            "name": "seizedCollateralAmount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "marketConfig",
      "serialization": "bytemuck",
      "repr": {
        "kind": "c",
        "packed": true
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "governanceAuthority",
            "type": "pubkey"
          },
          {
            "name": "status",
            "docs": [
              "Protocol status: 0=Active, 1=Paused, 2=WithdrawOnly"
            ],
            "type": "u8"
          },
          {
            "name": "poolCount",
            "type": "u16"
          },
          {
            "name": "pools",
            "type": {
              "array": [
                "pubkey",
                32
              ]
            }
          }
        ]
      }
    },
    {
      "name": "marketConfigInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "newGovernanceAuthority",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "operation",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "deposit",
            "fields": [
              {
                "name": "amount",
                "type": "u64"
              }
            ]
          },
          {
            "name": "withdraw",
            "fields": [
              {
                "name": "amount",
                "type": "u64"
              }
            ]
          },
          {
            "name": "borrow",
            "fields": [
              {
                "name": "amount",
                "type": "u64"
              }
            ]
          },
          {
            "name": "repay",
            "fields": [
              {
                "name": "amount",
                "type": "u64"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "operationsExecuted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "user",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "protocolFeesCollected",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "recipient",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "protocolPaused",
      "type": {
        "kind": "struct",
        "fields": []
      }
    },
    {
      "name": "protocolUnpaused",
      "type": {
        "kind": "struct",
        "fields": []
      }
    },
    {
      "name": "protocolWithdrawOnlyModeEnabled",
      "type": {
        "kind": "struct",
        "fields": []
      }
    },
    {
      "name": "repaid",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "userPosition",
      "serialization": "bytemuck",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "collateralAmount",
            "type": "u64"
          },
          {
            "name": "loanAmount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "withdrawn",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    }
  ]
};
