const hre = require("hardhat");
const delay = ms => new Promise(res => setTimeout(res, ms));

async function deployTokenAndMint(owner, to) { 
  const tokenFactory = await hre.ethers.getContractFactory("contracts/token.sol:Token");
  const tokenContract = await tokenFactory.connect(owner).deploy();
  console.log(`Token deployed to ${tokenContract.target}`);
  await (await tokenContract.connect(owner).addMinter(owner)).wait();
  await (await tokenContract.connect(owner).mint(hre.ethers.parseUnits("1000000", "ether"))).wait();
  await (await tokenContract.connect(owner).transfer(to, hre.ethers.parseUnits("1000000", "ether"))).wait();
  return tokenContract.target;
}

async function deployPBMAndFund(tokenAddress, owner, fundFrom, fundTo) {
  const tokenContract = await hre.ethers.getContractAt("contracts/tokenInterface.sol:Token", tokenAddress);
  const PBMFactory = await hre.ethers.getContractFactory("PBM");
  const PBMContract = await PBMFactory.connect(owner).deploy(tokenContract.target);
  console.log(`PBM deployed to ${PBMContract.target}`);
  await (await PBMContract.connect(owner).addAdmin(owner)).wait();
  // You need to add pay to address to the whitelist.
  await (await PBMContract.connect(owner).addWhiteList(owner)).wait();
  await (await PBMContract.connect(owner).addUser(fundTo)).wait();
  await (await tokenContract.connect(fundFrom).approve(PBMContract.target, hre.ethers.parseUnits("1000", "ether"))).wait();
  await (await PBMContract.connect(fundFrom).fundUser(fundTo, hre.ethers.parseUnits("1000", "ether"))).wait();

}

async function main(){
  const accounts = await hre.ethers.getSigners();
  const owner = accounts[0];
  const userB = accounts[1];
  const userC = accounts[2];
  tokenAddress = await deployTokenAndMint(owner, userB);
  await deployPBMAndFund(tokenAddress, owner, userB, userC);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
