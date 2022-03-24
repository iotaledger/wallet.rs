(self.webpackChunkdocs=self.webpackChunkdocs||[]).push([[692],{4236:function(e,t,r){"use strict";r.r(t),r.d(t,{frontMatter:function(){return l},contentTitle:function(){return i},metadata:function(){return s},toc:function(){return p},default:function(){return d}});var n=r(2122),a=r(9756),o=(r(7294),r(3905)),u=["components"],l={},i=void 0,s={unversionedId:"libraries/java/api/SignatureLockedDustAllowanceOutput",id:"libraries/java/api/SignatureLockedDustAllowanceOutput",isDocsHomePage:!1,title:"SignatureLockedDustAllowanceOutput",description:"Output type for deposits that enables an address to receive dust outputs. It can be consumed as an input like a",source:"@site/docs/libraries/java/api/SignatureLockedDustAllowanceOutput.mdx",sourceDirName:"libraries/java/api",slug:"/libraries/java/api/SignatureLockedDustAllowanceOutput",permalink:"/docs/libraries/java/api/SignatureLockedDustAllowanceOutput",editUrl:"https://github.com/iotaledger/wallet.rs/tree/dev/documentation/docs/libraries/java/api/SignatureLockedDustAllowanceOutput.mdx",version:"current",frontMatter:{}},p=[{value:"from(address, amount): SignatureLockedDustAllowanceOutput",id:"fromaddress-amount-signaturelockeddustallowanceoutput",children:[]},{value:"amount(): long",id:"amount-long",children:[]},{value:"address(): AddressWrapper",id:"address-addresswrapper",children:[]}],c={toc:p};function d(e){var t=e.components,r=(0,a.Z)(e,u);return(0,o.kt)("wrapper",(0,n.Z)({},c,r,{components:t,mdxType:"MDXLayout"}),(0,o.kt)("p",null,"Output type for deposits that enables an address to receive dust outputs. It can be consumed as an input like a\nregular SigLockedSingleOutput."),(0,o.kt)("h3",{id:"fromaddress-amount-signaturelockeddustallowanceoutput"},"from(address, amount): ",(0,o.kt)("a",{parentName:"h3",href:"#signaturelockeddustallowanceoutput"},"SignatureLockedDustAllowanceOutput")),(0,o.kt)("p",null,"Creates a new ",(0,o.kt)("inlineCode",{parentName:"p"},"SignatureLockedDustAllowanceOutput"),"."),(0,o.kt)("table",null,(0,o.kt)("thead",{parentName:"table"},(0,o.kt)("tr",{parentName:"thead"},(0,o.kt)("th",{parentName:"tr",align:null},"Parameter"),(0,o.kt)("th",{parentName:"tr",align:null},"Type"),(0,o.kt)("th",{parentName:"tr",align:null},"Description"))),(0,o.kt)("tbody",{parentName:"table"},(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"address"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"#addresswrapper"},"AddressWrapper")),(0,o.kt)("td",{parentName:"tr",align:null},"The address to set")),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"amount"),(0,o.kt)("td",{parentName:"tr",align:null},"long"),(0,o.kt)("td",{parentName:"tr",align:null},"The amount to set")))),(0,o.kt)("h3",{id:"amount-long"},"amount(): long"),(0,o.kt)("p",null,"Returns the amount of a ",(0,o.kt)("inlineCode",{parentName:"p"},"SignatureLockedDustAllowanceOutput"),"."),(0,o.kt)("h3",{id:"address-addresswrapper"},"address(): ",(0,o.kt)("a",{parentName:"h3",href:"#addresswrapper"},"AddressWrapper")),(0,o.kt)("p",null,"Returns the address of a ",(0,o.kt)("inlineCode",{parentName:"p"},"SignatureLockedDustAllowanceOutput"),"."))}d.isMDXComponent=!0},3905:function(e,t,r){"use strict";r.d(t,{Zo:function(){return p},kt:function(){return m}});var n=r(7294);function a(e,t,r){return t in e?Object.defineProperty(e,t,{value:r,enumerable:!0,configurable:!0,writable:!0}):e[t]=r,e}function o(e,t){var r=Object.keys(e);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(e);t&&(n=n.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),r.push.apply(r,n)}return r}function u(e){for(var t=1;t<arguments.length;t++){var r=null!=arguments[t]?arguments[t]:{};t%2?o(Object(r),!0).forEach((function(t){a(e,t,r[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(r)):o(Object(r)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(r,t))}))}return e}function l(e,t){if(null==e)return{};var r,n,a=function(e,t){if(null==e)return{};var r,n,a={},o=Object.keys(e);for(n=0;n<o.length;n++)r=o[n],t.indexOf(r)>=0||(a[r]=e[r]);return a}(e,t);if(Object.getOwnPropertySymbols){var o=Object.getOwnPropertySymbols(e);for(n=0;n<o.length;n++)r=o[n],t.indexOf(r)>=0||Object.prototype.propertyIsEnumerable.call(e,r)&&(a[r]=e[r])}return a}var i=n.createContext({}),s=function(e){var t=n.useContext(i),r=t;return e&&(r="function"==typeof e?e(t):u(u({},t),e)),r},p=function(e){var t=s(e.components);return n.createElement(i.Provider,{value:t},e.children)},c={inlineCode:"code",wrapper:function(e){var t=e.children;return n.createElement(n.Fragment,{},t)}},d=n.forwardRef((function(e,t){var r=e.components,a=e.mdxType,o=e.originalType,i=e.parentName,p=l(e,["components","mdxType","originalType","parentName"]),d=s(r),m=a,f=d["".concat(i,".").concat(m)]||d[m]||c[m]||o;return r?n.createElement(f,u(u({ref:t},p),{},{components:r})):n.createElement(f,u({ref:t},p))}));function m(e,t){var r=arguments,a=t&&t.mdxType;if("string"==typeof e||a){var o=r.length,u=new Array(o);u[0]=d;var l={};for(var i in t)hasOwnProperty.call(t,i)&&(l[i]=t[i]);l.originalType=e,l.mdxType="string"==typeof e?e:a,u[1]=l;for(var s=2;s<o;s++)u[s]=r[s];return n.createElement.apply(null,u)}return n.createElement.apply(null,r)}d.displayName="MDXCreateElement"}}]);