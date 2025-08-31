/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/flash_loan_receiver.json`.
 */
export type FlashLoanReceiver = {
  "address": "Agxw43dYHrUcCiJPAeTKe4QK4qfgQPoAPTWrwkQepiw7",
  "metadata": {
    "name": "flashLoanReceiver",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "executeOperation",
      "discriminator": [
        245,
        135,
        123,
        214,
        182,
        74,
        195,
        199
      ],
      "accounts": [
        {
          "name": "user",
          "docs": [
            "We manually check `is_signer` in the instruction logic."
          ],
          "writable": true
        },
        {
          "name": "userTokenAccount",
          "writable": true
        },
        {
          "name": "vaultTokenAccount",
          "writable": true
        },
        {
          "name": "mint"
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": [
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
  ],
  "errors": [
    {
      "code": 6000,
      "name": "userNotSigner",
      "msg": "The user account must be a signer."
    }
  ]
};
