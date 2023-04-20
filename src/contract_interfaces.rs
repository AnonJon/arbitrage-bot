use ethers::prelude::abigen;

abigen!(
    IERC20,
    r#"[
        function totalSupply() external view returns (uint256)
        function balanceOf(address account) external view returns (uint256)
        function transfer(address recipient, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function transferFrom( address sender, address recipient, uint256 amount) external returns (bool)
        function decimals() external view returns (uint8)
        function symbol() external view returns (string memory)
    ]"#,
);

abigen!(
    IUniswapV2Pair,
    r#"[
        function token0() external view returns (address)
        function token1() external view returns (address)
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
        function price0CumulativeLast() external view returns (uint)
        function price1CumulativeLast() external view returns (uint)
    ]"#,
);

abigen!(
    IBalancerVault,
    r#"[
        function getPool(bytes32 poolId) external view returns (address pair, uint8 tokens)
        function getPoolTokens(bytes32 poolId) external view returns (address[] memory tokens, uint256[] memory balances, uint256 lastChangeBlock)
    ]"#,
);

abigen!(
    IBalancerPool,
    r#"[
        function getPoolId() external view returns (bytes32)
    ]"#,
);
