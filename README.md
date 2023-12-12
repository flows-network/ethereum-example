# ethereum-example 
[Deploy this function on flows.network](#deploy-the-ethereum-example), and you will get a web service that using ethers to send transaction.

![image](https://i.imgur.com/o9bMy6J.png)

## Deploy the ethereum-example

To install this ethereum-example, we will use [flows.network](https://flows.network/), a serverless platform that makes deploying your own app quick and easy in just three steps.

## Prerequisite

You will need a wallet private key. If you do not already have one, use [Metamask](https://metamask.io/) to create it.

### Fork this repo and write your own code

Fork [this repo](https://github.com/flows-network/ethereum-example). 

### Deploy the code on flow.network

1. Sign up for an account for deploying flows on [flows.network](https://flows.network/). It's free.
2. Click on the "Create a Flow" button to start deploying the web service.
3. Authenticate the [flows.network](https://flows.network/) to access the `ethereum-example` repo you just forked. 

<img width="948" alt="image" src="https://i.imgur.com/AsXQyyl.png">

4. Click on the Advanced text and you will see more settings including branch and environment variables. In this example, we have one variable `PRIVATE_KEY` to fill in, which is the wallet private key.
The default network is Arbitrum sepolia. If you want to change network, you can set `RPC_NODE_URL` and `CHAIN_ID` variable.

<img width="899" alt="image" src="https://i.imgur.com/257iBGw.png">

5. Click the Deploy button to deploy your function.

### Configure SaaS integrations

After that, the flows.network will direct you to configure the SaaS integration required by your flow. Here we can see: there is no SaaS needs to be connected since it's a lambda service. Just click the Check button to see your flow details.

<img width="964" alt="image" src="https://user-images.githubusercontent.com/45785633/226959151-0e8a159a-02b3-4130-b7b5-8831b65c8d75.png">

## Try this demo

After the flow function's status becomes `ready`, you will see a link under the Lambda Endpoint.

In the ethereum example, it implements three types of web3 service. You can use different paths of URLs to change services.

![image](https://i.imgur.com/ZINnavr.png)

### Sign-tx

You just provide basic information, sign-tx service can help you sign and send transactions.

Copy and paste the endpoint URL to your browser and add `/sign-tx?address_to=0xf04c6a55F0fdc0A5490d83Be69A7A675912A5AB3&value=10000000000000000` to send 0.01 ETH to `0xf04c6a55F0fdc0A5490d83Be69A7A675912A5AB3`. Then you can see the transaction hash.

![image](https://i.imgur.com/o9bMy6J.png)

If you want to send a transaction with a `data` parameter, you can add a new query parameter named `data` to send hex encode bytes.

### PBM-pay

Before the demo, you can follow the README of [pbm-pilot](https://github.com/coda-bridge/pbm-pilot) to create a hardhat project.
Upload your ERC 20 token contract  or [stablecoin](https://github.com/coda-bridge/stablecoin)'s  token.sol to `hardhat/contracts`
Finally copy `ethereum-example.js` to `hardhat/script`, then use `npx hardhat run script/ethereum-example.js --network sepolia` to run the script.

The script will do the following things.
1. Deploy an ERC 20 token contract and PBM contract.
2. Mint 1000000 token to userB.
3. userB use pbm fund 1000 token to userC

After running the script, you need to set the PBM contract address to `CONTRACT_ADDRESS` in flows.network setting configuration and ensure the `PRIVATE_KEY` is userC.
![image](https://i.imgur.com/5Agyp3N.png)

PBM-pay service will call the PBM contract pay function to transfer 10 tokens to someone.

Copy and paste the endpoint URL to your browser and add `/pbm-pay?pay-to=0x70655c1c0D7Ef1273D17E5610a2079E7465F0FF6` to let PBM contract send 10 tokens to `0x70655c1c0D7Ef1273D17E5610a2079E7465F0FF6`. Then you can see the transaction hash.\
(Ensure the pay-to address has been added to the PBM contract whitelist)

![image](https://i.imgur.com/xPsd3JX.png)

### Gen-key

Gen-key service will random generate an ethereum account. If you want to generate an account from mnemonic, you and add `phrase` to query parameter.
Copy and paste the endpoint URL to your browser and add `/gen-key`. Then you can see the account address.

![image](https://i.imgur.com/ihZ3Yyn.png)

The Running log will show the address and the private key.

![image](https://i.imgur.com/StSqkK3.png)

> [flows.network](https://flows.network/) is still in its early stages. We would love to hear your feedback!

## Others


To build locally, make sure you have intsalled Rust and added `wasm32-wasi` target.

```
cargo build target wasm32-wasi --release
```
