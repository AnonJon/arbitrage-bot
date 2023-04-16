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
        event Transfer(address indexed from, address indexed to, uint256 value)
        event Approval(address indexed owner, address indexed spender, uint256 value)
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
        function kLast() external view returns (uint)
        function mint(address to) external returns (uint liquidity)
        function burn(address to) external returns (uint amount0, uint amount1)
        function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external
        function skim(address to) external
        function sync() external
        event Sync(uint112 reserve0, uint112 reserve1)
    ]"#,
);
