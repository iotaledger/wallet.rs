/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
module.exports = {
    preset: 'ts-jest',
    testEnvironment: 'node',
    testMatch: ['<rootDir>/tests/**/*.(test|spec).ts'],
    moduleNameMapper: {
        'index.node': '<rootDir>/build/Release/index.node',
    },
};
