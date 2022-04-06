// @ts-ignore
const addon = require('../build/Release/index.node');

console.log('Add on', addon);

const {
    initLogger,
    sendMessage,
    messageHandlerNew,
    listen, 
    eventListenerNew, 
    removeEventListeners
} = addon;

export {
    initLogger,
    sendMessage,
    messageHandlerNew,
    listen, 
    eventListenerNew, 
    removeEventListeners
};
