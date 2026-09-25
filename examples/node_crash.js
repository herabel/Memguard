// To start this one you should be in main folder and use:
//
// NODE_OPTIONS="--require ./sdk/agent.js" node examples/node_crash.js

console.log('[test] Triggering native abort in Node.js process...');
process.abort();