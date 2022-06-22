/**
 * * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

module.exports = {
  docs: [
    {
      type: 'doc',
      label: 'Welcome',
      id: 'welcome',
    },
    {
      type: 'doc',
      label: 'Overview',
      id: 'overview',
    },
    {
      type:'category',
      label: 'Getting Started',
      items:[
        {
          type:'doc',
          label: 'Rust',
          id:'getting_started/rust'
        },
        {
          type:'doc',
          label: 'Node.js',
          id:'getting_started/nodejs'
        },
        {
          type:'doc',
          label: 'Python',
          id:'getting_started/python'
        },
        {
          type:'doc',
          label: 'Java',
          id:'getting_started/java'
        },
        {
          type:'doc',
          label: 'Java for Android',
          id:'getting_started/java_for_android'
        },
      ]
    },
    {
      type: 'category',
      label: 'Explanations',
      items:[
        {
          type: 'doc',
          label: 'The Library in a Nutshell',
          id: 'explanations/nutshell',
        }
      ]
    },
    {
      type: 'category',
      label: 'Examples',
      items:[
        {
          type:'doc',
          label: 'Rust',
          id:'examples/rust'
        },
        {
          type:'doc',
          label: 'Node.js',
          id:'examples/nodejs'
        },
        {
          type:'doc',
          label: 'Python',
          id:'examples/python'
        },
        {
          type:'doc',
          label: 'Java',
          id:'examples/java'
        },
      ]
    },
    {
      type:'category',
      label: 'Reference',
      items:[
        {
          type:'doc',
          label: 'Library Specifications',
          id:'reference/specifications'
        },
        {
          type:'doc',
          label: 'Rust API',
          id:'reference/rust'
        },
        {
          type:'doc',
          label: 'Node.js API',
          id:'reference/nodejs'
        },
        {
          type:'doc',
          label: 'Python API',
          id:'reference/python'
        },
        {
          type:'doc',
          label: 'Java API',
          id:'reference/java'
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