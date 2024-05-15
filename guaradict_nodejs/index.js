"use strict";

const { createDriver, connect, disconnect, set, get } = require('./index.node');

let currentIndex

global.onEvent = function (event) {
    console.log('Event:', event);
};

async function main() {
    const driver = createDriver('127.0.0.1:13141');
    try {
        const index = await connect.call(driver);

        await set.call(driver, index, "my-key", "val");
        const response = await get.call(driver, index, "my-key");
        console.log('GET my-key:', response);

        await disconnect.call(driver, index);
    } catch (err) {
        console.error('Error:', err);
    }
}

main();
