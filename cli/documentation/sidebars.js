/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
    tutorialSidebar: [
      {
        type: 'doc',
        id: 'welcome',
        label: 'Welcome'
      },
      {
        type: 'doc',
        id: 'installation',
        label: 'Installation'
      },
      {
        type: 'doc',
        id: 'account_manager',
        label: 'Account manager'
      },
      {
        type: 'doc',
        id: 'account',
        label: 'Account'
      },
      {
        type: 'doc',
        id: 'step_by_step',
        label: 'Step by step'
      },
    ],
};

module.exports = sidebars;
