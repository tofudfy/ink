# ERC20 Contract Tutorial

## Contract Compile

The example of the ERC20 contract can be find in [Github/paritytech/ink](<https://github.com/paritytech/ink/tree/ee42309d830b1b4b4f3874c2f53c3f5d6ae9b47f/examples/erc20>) repo.

Clone the repo and compile the ERC20 contract locally

<img src="./imgs/complile.png" alt="complile" style="zoom:80%;" />

## Node Setup

Build and Run the [substrate-contracts-node](<https://github.com/paritytech/substrate-contracts-node>) locally

```shell
./target/release/substrate-contracts-node --dev --tmp
```

Open the front-end UI ([Polkadot.js](<https://polkadot.js.org/apps/#/contracts>) or [Canvas UI](<https://paritytech.github.io/canvas-ui/#/>)) and connect to the local node



## Deploy and Initialize Contract

Upload the file `erc20.contract`

<img src="./imgs/Deploy.png" alt="Deploy" style="zoom:30%;" />

Initial 1000 supply to Alice (default account)

<img src="./imgs/Initial.png" alt="Initial" style="zoom:30%;" />

## Balance

Check the balance after deploying erc20 contract successfully

<img src="./imgs/Balance.png" alt="Balance" style="zoom:30%;" />

## Transfer

Alice transfer 100 to Bob

<img src="./imgs/transfer.png" alt="transfer" style="zoom:30%;" />

Check the balance after transfer

<img src="./imgs/transfer_balance.png" alt="transfer_balance" style="zoom:30%;" />

## Approve and Allowance

Alice approve 300 amount of allowance to Charlie

<img src="./imgs/allowance.png" alt="allowance" style="zoom:30%;" />

Check the allowance

<img src="./imgs/allowance_check.png" alt="allowance_check" style="zoom:30%;" />

Charlie transfer 200 from alice to bob

<img src="./imgs/transfer_from.png" alt="transfer_from" style="zoom:30%;" />

Check the remaining allowance form Alice to Charlie

<img src="./imgs/allowance_remain.png" alt="allowance_remain" style="zoom:30%;" />

Check the balance again

<img src="./imgs/transfer_from_balance.png" alt="transfer_from_balance" style="zoom:30%;" />