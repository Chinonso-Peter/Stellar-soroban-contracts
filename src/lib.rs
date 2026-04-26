// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./SecurityEvents.sol";

contract GovernanceManager is SecurityEvents {
    mapping(bytes32 => bool) public executedProposals;

    /// @notice Marks a governance proposal as executed and emits the action for audit tracking.
    /// @dev Reverts when the proposal ID has already been executed to prevent replayed actions.
    function executeProposal(
        bytes32 proposalId,
        string calldata action
    ) external {
        require(!executedProposals[proposalId], "Proposal already executed");

        executedProposals[proposalId] = true;

        emit GovernanceActionExecuted(
            msg.sender,
            proposalId,
            action,
            block.timestamp
        );
    }
}
