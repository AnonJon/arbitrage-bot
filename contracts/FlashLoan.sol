pragma solidity ^0.8.0;

import "@aave/protocol-v2/contracts/flashloan/base/FlashLoanReceiverBase.sol";
import "@aave/protocol-v2/contracts/interfaces/ILendingPoolAddressesProvider.sol";

contract FlashLoanExample is FlashLoanReceiverBase {
    address constant DAI_ADDRESS = 0x6B175474E89094C44Da98b954EedeAC495271d0F;

    constructor(ILendingPoolAddressesProvider _addressProvider) FlashLoanReceiverBase(_addressProvider) {}

    function executeFlashLoan(uint256 amount) public {
        // Get the LendingPool contract address
        address lendingPool = ADDRESS_PROVIDER.getLendingPool();

        // Request the flash loan
        bytes memory data = "";
        ILendingPool(lendingPool).flashLoan(address(this), DAI_ADDRESS, amount, data);
    }

    function executeOperation(
        address[] calldata assets,
        uint256[] calldata amounts,
        uint256[] calldata premiums,
        address initiator,
        bytes calldata params
    ) external override returns (bool) {
        // Logic for your use case

        // Repay the loan and the fee
        for (uint256 i = 0; i < assets.length; i++) {
            uint256 amountOwing = amounts[i] + premiums[i];
            ERC20(assets[i]).approve(address(LENDING_POOL), amountOwing);
        }

        return true;
    }
}
