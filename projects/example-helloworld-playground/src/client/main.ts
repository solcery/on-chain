/**
 * Hello world
 */

import {
  establishConnection,
  establishPayer,
  checkProgram,
  storeNumber,
  reportGreetings,
} from './hello_world';

const prompt = require('prompt-sync')();
async function main() {
  console.log("Let's say hello to a Solana account...");

  // Establish connection to the cluster
  await establishConnection();

  // Determine who pays for the fees
  await establishPayer();

  // Check if the program has been deployed
  await checkProgram();

  const numb: number = +prompt("What is your number?");
  
  console.log("Hey, your number is ", numb);
  // Say hello to an account
  await storeNumber(numb);

  // Find out how many times that account has been greeted
  //await reportGreetings();

  //console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
