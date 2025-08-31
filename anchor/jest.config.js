// anchor/jest.config.js
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  // Jest 会在这个目录下寻找测试文件
  roots: ['<rootDir>/tests'],
  // 增加测试超时时间，因为与真实网络交互可能会慢
  testTimeout: 90000,
}
