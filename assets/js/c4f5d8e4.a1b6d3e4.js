(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[4195],{5502:function(e,t,n){"use strict";var a=n(7294).createContext(void 0);t.Z=a},6266:function(e,t,n){"use strict";var a=n(7294),r=n(5502);t.Z=function(){var e=(0,a.useContext)(r.Z);if(null==e)throw new Error('"useThemeContext" is used outside of "Layout" component. Please see https://docusaurus.io/docs/api/themes/configuration#usethemecontext.');return e}},4513:function(e,t,n){"use strict";n.r(t),n.d(t,{default:function(){return y}});var a=n(7294),r=n(8001),i=(n(1384),n(6832)),s=n(6010),l=n(2122),c=n(5977),o="header_1hEk",u="headerTitle_2ZSG",m="button_tkZj",d="icon_13V2",h="body_2-wU",p=[{title:"Learn",link:"docs/welcome",description:a.createElement(a.Fragment,null,"Learn the Basics about the IOTA Wallet.rs Library and how it works behind the scenes.")},{title:"Build",link:"docs/libraries/overview",description:a.createElement(a.Fragment,null,"Follow our tutorial to build your own IOTA  application. The IOTA Wallet.rs Library supports Rust, Python and Javascript.")},{title:"Participate",link:"docs/contribute",description:a.createElement(a.Fragment,null,"You want to be a part of the IOTA mission? Join the IOTA community or join the IOTA Libraries X-Team.")}];function v(e){var t=e.title,n=e.link,r=e.description,i=(0,a.useState)(!1),l=i[0],p=i[1],v=(0,c.k6)();return a.createElement("div",{className:"col col--4 margin-vert--md"},a.createElement("div",{className:(0,s.Z)("card padding--lg"),onClick:function(e){e.preventDefault(),v.push(n)},onMouseOver:function(){return p(!0)},onMouseOut:function(){return p(!1)}},a.createElement("div",{className:(0,s.Z)(o)},a.createElement("span",{className:(0,s.Z)(u)},t),a.createElement("div",{href:n,className:(0,s.Z)(m)},a.createElement("span",{className:(0,s.Z)("material-icons",d)},"navigate_next"))),a.createElement("div",{className:(0,s.Z)("headline-stick",{"size-m":l,"size-s":!l})}),a.createElement("div",{className:(0,s.Z)(h)},r)))}var E=function(){return a.createElement("div",{className:"container padding--xl"},a.createElement("div",{className:"section-header grey text--center margin-bottom--sm"},"Get started, right away"),a.createElement("div",{className:"row"},p.map((function(e,t){return a.createElement(v,(0,l.Z)({key:t},e))}))))},f=n(1402),g="heading_3OS9",Z="headingImage_4-y5",N="headingTitle_2e_T";var b=function(){var e=(0,i.Z)().siteConfig;return a.createElement("div",{className:(0,s.Z)(g)},a.createElement("img",{className:(0,s.Z)(Z),src:(0,f.Z)("/img/libraries.png")}),a.createElement("div",null,a.createElement("h1",{className:(0,s.Z)(N)},e.title),a.createElement("span",{className:(0,s.Z)("grey","section-header")},e.tagline)))},k="header_2pud";var w=function(){return a.createElement("header",{className:(0,s.Z)("padding-top--xl",k)},a.createElement(b,null),a.createElement(E,null))};function y(){var e=(0,i.Z)().siteConfig,t=e.title,n=e.tagline;return a.createElement(r.Z,{title:t,description:n},a.createElement(w,null))}}}]);