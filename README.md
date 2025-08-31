# Solana 高级 DeFi 借贷协议 (全栈实现)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Powered by Anchor](https://img.shields.io/badge/Powered%20by-Anchor-blue.svg)](https://www.anchor-lang.com/) [![Frontend: React & Next.js](https://img.shields.io/badge/Frontend-React%20%26%20Next.js-cyan.svg)](https://nextjs.org/)

这是一个基于 Solana 和 Anchor 框架构建的企业级全栈 DeFi 借贷协议。项目完整实现了一个类似 Aave 或 Compound
的去中心化借贷市场，支持用户存入资产作为抵押品以赚取利息，并借出其他资产。该协议包含了动态利率模型、价格预言机集成、链上清算、闪电贷、信用委托和批量交易等多种高级
DeFi 功能，旨在提供一个安全、灵活且功能丰富的链上金融解决方案。

## ✨ 核心功能

- **多资产借贷池**:
    - 协议支持通过治理添加多个独立的资产池（如 USDC, SOL），每个池都有自己独特的风险参数和利率模型。
- **核心借贷操作**:
    - **存款 (Deposit)**: 用户存入资产作为抵押品，并开始赚取浮动存款利息。
    - **取款 (Withdraw)**: 用户可以随时取回其抵押品，前提是其账户健康度保持在安全水平之上。
    - **借款 (Borrow)**: 用户可以根据其抵押品价值，借出池中的资产。
    - **还款 (Repay)**: 用户可以随时偿还部分或全部借款。
- **动态利率模型**:
    - 采用“拐点 (Kink)”利率模型，根据每个资产池的**资金利用率**动态调整借款和存款年化收益率 (APY)，以激励市场平衡。
- **链上清算机制**:
    - 当用户的债务价值超过其抵押品价值的**清算阈值**时，其头寸将面临被清算的风险。
    - 任何第三方（清算人）都可以偿还用户的部分债务，并以折扣价获得其部分抵押品，从而维护协议的偿付能力。
- **高级 DeFi 功能**:
    - **闪电贷 (Flash Loans)**: 允许开发者在一个原子交易内无抵押借出池中资产，并在交易结束前归还本息，为套利和复杂 DeFi
      操作提供了可能。
    - **信用委托 (Credit Delegation)**: 允许用户将其头寸的借贷能力委托给另一个地址，而无需转移底层抵押品，极大地提高了资本效率。
    - **批量操作 (Batch Operations)**: 支持用户在一个交易中捆绑多个操作（存、取、借、还），以优化 Gas 费用并实现复杂策略。
- **强大的治理模块**:
    - 协议的核心参数（如风险参数、利率模型、新资产上市）均由**治理权限**控制。
    - 支持**暂停/恢复**协议或将其设置为**仅限提款模式**，以应对市场波动或进行安全升级。

## 🛠️ 技术栈

- **智能合约**: Rust, **Anchor Framework v0.29+**
- **核心优化**: **`zero-copy`** 反序列化，高精度数学库 `U192`
- **区块链**: Solana
- **预言机**: **Pyth Network**, **Chainlink** (支持回退机制)
- **前端框架**: React, Next.js
- **UI**: Shadcn/UI, Tailwind CSS
- **异步状态管理**: **TanStack Query (React Query)**
- **钱包集成**: Solana Wallet Adapter
- **测试**: TypeScript, Mocha, Chai, **模拟预言机**

## 📂 项目结构

```
.
├── anchor/                  # Anchor 项目
│   ├── programs/lending_protocol/ # 借贷智能合约源码 (lib.rs)
│   └── tests/lending_protocol.ts  # 集成测试脚本
├── app/                     # Next.js 前端应用
│   ├── components/lending/
│   │   ├── lendingProtocol-data-access.ts # 核心数据访问层 (React Hooks)
│   │   └── lendingProtocol-ui.tsx         # 所有 UI 组件
│   └── app/lending/page.tsx             # 功能主页/容器组件
├── package.json
└── README.md
```

## 🚀 快速开始

### 先决条件

- [Node.js v18 或更高版本](https://nodejs.org/en/)
- [Rust 工具链](https://www.rust-lang.org/tools/install)
- [Solana CLI v1.17 或更高版本](https://docs.solana.com/cli/install)
- [Anchor CLI v0.29 或更高版本](https://www.anchor-lang.com/docs/installation)

### 1. 部署智能合约

1. **启动本地验证器**:
   ```bash
   solana-test-validator
   ```
2. **构建并部署合约**: 在项目根目录下，打开另一个终端窗口运行：
   ```bash
   # 注意：测试脚本依赖于名为 `mock_writer` 的另一个程序用于模拟预言机
   # 请确保所有依赖的程序都已正确构建和部署
   anchor build && anchor deploy
   ```
3. **记录程序 ID**: 部署成功后，复制输出的程序 ID。

### 2. 运行前端应用

1. **更新配置**: 将上一步获取的程序 ID 和在本地部署的资产 Mint/Pyth 地址更新到前端代码中（特别是 `lendingProtocol-ui.tsx`
   的 `GovernanceCard` 组件）。
2. **安装依赖**:
   ```bash
   npm install
   ```
3. **启动开发服务器**:
   ```bash
   npm run dev
   ```
4. 在浏览器中打开 `http://localhost:3000` 即可访问 dApp。

## ✅ 运行测试

我们提供了全面的集成测试，覆盖了治理、核心借贷流程、高级功能和清算场景。

```bash
anchor test
```

**注意**: 测试套件使用了一个**模拟预言机合约 (`MockWriter`)** 来在本地环境中创建和更新 Pyth 价格账户，这是成功运行测试的关键。

## 📜 智能合约深度解析

智能合约 (`programs/lending_protocol/src/lib.rs`) 是一个高度模块化和安全优化的系统。

- **`zero-copy` 优化**: 所有主要的协议状态账户（`MarketConfig`, `AssetPool`, `UserPosition`）都使用了 `zero-copy`
  ，这对于需要频繁读写的复杂 DeFi 协议至关重要，可以显著降低计算单元 (CU) 消耗和交易延迟。
- **多预言机集成**: 协议的 `oracle` 模块抽象了价格获取逻辑，支持从 Pyth 和 Chainlink
  获取价格，并提供了回退机制。它还包含了对价格时效性、置信区间等关键安全指标的检查。
- **利息累计 (`accrue_interest`)**: 这是协议的核心引擎。在几乎所有用户交互之前，都会调用此函数来根据时间差和当前资金利用率计算并累积利息。这确保了协议的账本始终保持最新状态。
- **高精度数学 (`U192`)**: 在所有涉及价值计算（例如，抵押品价值、健康度、清算奖金）的地方，都使用了 `U192`
  大数库，以避免在乘以价格和除以精度因子时发生溢出或精度损失。
- **跨池清算 (`liquidate`)**: `liquidate` 指令的上下文设计允许清算人使用一种资产（偿还借款池的资产）来清算另一种资产（获取抵押品池的资产），这是现代借贷协议的标志性功能。

## 🖥️ 前端架构深度解析

前端应用 (`app/`) 采用了先进的 React 架构，实现了逻辑与视图的高度分离。

- **数据访问层 (`lendingProtocol-data-access.ts`)**:
    - **分层 Hooks**:
        - `useLendingProgram`: 管理全局状态（`MarketConfig`）和治理操作。
        - `useLendingPool`: 负责**单个**资产池的所有数据查询和交互逻辑，包括池本身的状态、用户在该池中的头寸以及所有借贷操作。
        - `useUserBalance`: 一个可复用的原子化 Hook，用于查询任意代币的余额。
    - **智能状态管理**: 深度整合 **`TanStack Query`**，通过依赖查询和智能的 `invalidateQueries`
      策略，确保了在用户执行任何操作后，所有受影响的链上数据都能及时、高效地在 UI 上得到更新。

- **UI 组件层 (`lendingProtocol-ui.tsx`)**:
    - **组件化**: UI 被拆分为逻辑清晰的组件，如治理面板 (`GovernanceCard`)、资产池列表 (`AssetPoolList`)
      和资产池交互卡片 (`AssetPoolCard`)。
    - **上下文感知 UI**:
        - **协议状态**: 如果协议未初始化，UI 会引导治理员进行初始化。
        - **用户状态**: `AssetPoolCard` 会检查用户是否已在池中创建头寸。如果没有，会显示“创建头寸”按钮；如果有，则显示完整的存、取、借、还操作面板。
    - **数据格式化**: 所有从链上获取的原始 `u64` 金额，在展示给用户前都会根据代币的 `decimals` 进行格式化，提供了友好的用户体验。

## 📄 许可证

本项目采用 [MIT 许可证](https://opensource.org/licenses/MIT)。