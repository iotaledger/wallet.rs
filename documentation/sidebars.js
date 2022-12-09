/**
 * * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

module.exports = {
  docs: [{
      type: 'doc',
      id: 'welcome',
    },
      {

          type: 'category',
          label: 'Overview',
          items:[
              'explanations/library_overview',
              'explanations/account_approaches',
          ]
      },
    {
      type: "category",
      label: 'Getting Started',

      link: {
          type: "doc",
          id:'getting_started/getting_started'

      },
      items: [
          {
              type: "doc",
              id: "getting_started/rust",
              label: 'Rust'
          },
          {
              type: "doc",
              id: "getting_started/nodejs",
              label: 'Node.js'
          },
          {
              type: "doc",
              id: "getting_started/java",
              label: 'Java'
          },
          {
              type: "doc",
              id: "getting_started/python",
              label: 'Python'
          },
      ]
    },
    {
      type: 'category',
      label: 'How To',
      items:[
          'how_tos/run_how_tos',
          {
              type: "category",
              label: 'Accounts and Addresses',
              items:[
                  {
                      type: 'autogenerated',
                      dirName: 'how_tos/accounts_and_addresses'
                  }
              ]
          },
          {
              type: "category",
              label: 'Outputs and Transactions',
              items:[
                  {
                      type: 'autogenerated',
                      dirName: 'how_tos/outputs_and_transactions'
                  }
              ]
          },
          {
              type: "category",
              label: 'Native Tokens',
              items:[
                  {
                      type: 'autogenerated',
                      dirName: 'how_tos/native_tokens'
                  }
              ]
          },

          {
              type: "category",
              label: 'NFTs',
              items:[
                  {
                      type: 'autogenerated',
                      dirName: 'how_tos/NFT'
                  }
              ]
          },
          'how_tos/more_examples',
          'how_tos/exchange_guide'
      ]
    },
    {

      type: 'category',
      label: 'API Reference',
      items:[
          {
              type: 'doc',
              id: 'references/rust_api_reference',
              label: 'Rust'
          },
          {
              type: 'category',
              label: 'Node.js',
              link:{
                  type: "doc",
                  id: "references/nodejs/api_ref",
              },
              items:[
                  {
                      type: "category",
                      label: "Classes",
                      items:[
                          {
                              type: 'autogenerated',
                              dirName: 'references/nodejs/classes'
                          },
                      ]
                  },
                  {
                      type: "category",
                      label: "Enums",
                      items:[
                          {
                              type: 'autogenerated',
                              dirName: 'references/nodejs/enums'
                          },
                      ]
                  },
                  {
                      type: "category",
                      label: "Interfaces",
                      items:[
                          {
                              type: 'autogenerated',
                              dirName: 'references/nodejs/interfaces'
                          },
                      ]
                  },
              ]
          },
          {
              type: 'category',
              label: 'Java',
              items:[
                  {
                      type:"autogenerated",
                      dirName:'references/java'
                  }
              ]
          },
          {
              type: 'category',
              label: 'Python',
              items:[
                  {
                      type:"autogenerated",
                      dirName:'references/python/iota_wallet'
                  }
              ]
          },
      ]
    },
    {
      type: 'doc',
      id: 'troubleshooting',
      label: 'Troubleshooting'
    },
    {
      type: 'doc',
      id: 'contribute',
      label: 'Contribute',
    }
  ]
};