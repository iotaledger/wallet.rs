(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[3412],{104:function(e,r,t){"use strict";t.r(r),t.d(r,{frontMatter:function(){return d},contentTitle:function(){return i},metadata:function(){return l},toc:function(){return o},default:function(){return u}});var a=t(2122),n=t(9756),p=(t(7294),t(3905)),s=["components"],d={},i=void 0,l={unversionedId:"libraries/java/api/AddressWrapper",id:"libraries/java/api/AddressWrapper",isDocsHomePage:!1,title:"AddressWrapper",description:"from(address, bech32_hrp): AddressWrapper",source:"@site/docs/libraries/java/api/AddressWrapper.mdx",sourceDirName:"libraries/java/api",slug:"/libraries/java/api/AddressWrapper",permalink:"/docs/libraries/java/api/AddressWrapper",editUrl:"https://github.com/iotaledger/wallet.rs/tree/dev/documentation/docs/libraries/java/api/AddressWrapper.mdx",version:"current",frontMatter:{}},o=[{value:"from(address, bech32_hrp): AddressWrapper",id:"fromaddress-bech32_hrp-addresswrapper",children:[]},{value:"parse(address): AddressWrapper",id:"parseaddress-addresswrapper",children:[]},{value:"toBech32(): String",id:"tobech32-string",children:[]}],c={toc:o};function u(e){var r=e.components,t=(0,n.Z)(e,s);return(0,p.kt)("wrapper",(0,a.Z)({},c,t,{components:r,mdxType:"MDXLayout"}),(0,p.kt)("h3",{id:"fromaddress-bech32_hrp-addresswrapper"},"from(address, bech32_hrp): ",(0,p.kt)("a",{parentName:"h3",href:"#addresswrapper"},"AddressWrapper")),(0,p.kt)("p",null,"Create an Address based on its address and Bech segments"),(0,p.kt)("table",null,(0,p.kt)("thead",{parentName:"table"},(0,p.kt)("tr",{parentName:"thead"},(0,p.kt)("th",{parentName:"tr",align:null},"Parameter"),(0,p.kt)("th",{parentName:"tr",align:null},"Type"),(0,p.kt)("th",{parentName:"tr",align:null},"Description"))),(0,p.kt)("tbody",{parentName:"table"},(0,p.kt)("tr",{parentName:"tbody"},(0,p.kt)("td",{parentName:"tr",align:null},"address"),(0,p.kt)("td",{parentName:"tr",align:null},"String"),(0,p.kt)("td",{parentName:"tr",align:null},"The Address segment")),(0,p.kt)("tr",{parentName:"tbody"},(0,p.kt)("td",{parentName:"tr",align:null},"bech32_hrp"),(0,p.kt)("td",{parentName:"tr",align:null},"String"),(0,p.kt)("td",{parentName:"tr",align:null},"the bech segment")))),(0,p.kt)("h3",{id:"parseaddress-addresswrapper"},"parse(address): ",(0,p.kt)("a",{parentName:"h3",href:"#addresswrapper"},"AddressWrapper")),(0,p.kt)("p",null,"parse a fully functional address string into an AddressWrapper"),(0,p.kt)("table",null,(0,p.kt)("thead",{parentName:"table"},(0,p.kt)("tr",{parentName:"thead"},(0,p.kt)("th",{parentName:"tr",align:null},"Parameter"),(0,p.kt)("th",{parentName:"tr",align:null},"Type"),(0,p.kt)("th",{parentName:"tr",align:null},"Description"))),(0,p.kt)("tbody",{parentName:"table"},(0,p.kt)("tr",{parentName:"tbody"},(0,p.kt)("td",{parentName:"tr",align:null},"address"),(0,p.kt)("td",{parentName:"tr",align:null},"String"),(0,p.kt)("td",{parentName:"tr",align:null},"The address we will parse")))),(0,p.kt)("h3",{id:"tobech32-string"},"toBech32(): String"),(0,p.kt)("p",null,"Get the bech segment of the address"))}u.isMDXComponent=!0},3905:function(e,r,t){"use strict";t.d(r,{Zo:function(){return o},kt:function(){return m}});var a=t(7294);function n(e,r,t){return r in e?Object.defineProperty(e,r,{value:t,enumerable:!0,configurable:!0,writable:!0}):e[r]=t,e}function p(e,r){var t=Object.keys(e);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);r&&(a=a.filter((function(r){return Object.getOwnPropertyDescriptor(e,r).enumerable}))),t.push.apply(t,a)}return t}function s(e){for(var r=1;r<arguments.length;r++){var t=null!=arguments[r]?arguments[r]:{};r%2?p(Object(t),!0).forEach((function(r){n(e,r,t[r])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(t)):p(Object(t)).forEach((function(r){Object.defineProperty(e,r,Object.getOwnPropertyDescriptor(t,r))}))}return e}function d(e,r){if(null==e)return{};var t,a,n=function(e,r){if(null==e)return{};var t,a,n={},p=Object.keys(e);for(a=0;a<p.length;a++)t=p[a],r.indexOf(t)>=0||(n[t]=e[t]);return n}(e,r);if(Object.getOwnPropertySymbols){var p=Object.getOwnPropertySymbols(e);for(a=0;a<p.length;a++)t=p[a],r.indexOf(t)>=0||Object.prototype.propertyIsEnumerable.call(e,t)&&(n[t]=e[t])}return n}var i=a.createContext({}),l=function(e){var r=a.useContext(i),t=r;return e&&(t="function"==typeof e?e(r):s(s({},r),e)),t},o=function(e){var r=l(e.components);return a.createElement(i.Provider,{value:r},e.children)},c={inlineCode:"code",wrapper:function(e){var r=e.children;return a.createElement(a.Fragment,{},r)}},u=a.forwardRef((function(e,r){var t=e.components,n=e.mdxType,p=e.originalType,i=e.parentName,o=d(e,["components","mdxType","originalType","parentName"]),u=l(t),m=n,f=u["".concat(i,".").concat(m)]||u[m]||c[m]||p;return t?a.createElement(f,s(s({ref:r},o),{},{components:t})):a.createElement(f,s({ref:r},o))}));function m(e,r){var t=arguments,n=r&&r.mdxType;if("string"==typeof e||n){var p=t.length,s=new Array(p);s[0]=u;var d={};for(var i in r)hasOwnProperty.call(r,i)&&(d[i]=r[i]);d.originalType=e,d.mdxType="string"==typeof e?e:n,s[1]=d;for(var l=2;l<p;l++)s[l]=t[l];return a.createElement.apply(null,s)}return a.createElement.apply(null,t)}u.displayName="MDXCreateElement"}}]);