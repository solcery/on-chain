/**
 * Hello world
 */

import {
  establishConnection,
  establishPayer,
  checkProgram,
  changeNumber,
  reportGreetings,
  executeImpact,
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

  //const operation: number = +prompt("What to do? 0 = add, 1 = sub");
  //const numb: number = +prompt("What is your number?");
  
  // Say hello to an account
  //await changeNumber(operation, numb);
  await executeImpact();
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
