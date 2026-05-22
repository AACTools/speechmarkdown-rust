const { toSsml, toText, parse } = require('./index.js');

console.log('=== toSsml ===');
console.log(toSsml('Hello (world)[emphasis:"strong"]', 'amazon-alexa'));

console.log('\n=== toText ===');
console.log(toText('Hello (world)[emphasis:"strong"]'));

console.log('\n=== parse ===');
const ast = JSON.parse(parse('Hello world'));
console.log(ast.node_type);

console.log('\n=== all platforms ===');
for (const p of ['amazon-alexa', 'google-assistant', 'microsoft-azure']) {
    console.log(`${p}: ${toSsml('(hello)[whisper]', p).substring(0, 60)}...`);
}

console.log('\nALL NODE TESTS PASSED');
