import{r as e,t}from"./rolldown-runtime-QTnfLwEv.js";import{A as n,C as r,E as i,O as a,P as o,S as s,T as c,v as l,w as u,x as d}from"./vendor-lit-MQ4DzmMc.js";import{o as f,s as p,t as m}from"./vendor-shared-62wh2Xkx.js";import{v as h}from"./index-NFRVgqV2.js";import{i as g}from"./dist-BqHmxVdQ.js";import{n as ee}from"./dist-CjVcNP8S.js";var _=class{constructor(e){this.streetAddress=e?.streetAddress,this.extendedAddress=e?.extendedAddress,this.locality=e?.locality,this.region=e?.region,this.postalCode=e?.postalCode,this.countryCodeAlpha2=e?.countryCodeAlpha2}},v=class{constructor(e){this.email=e?.email,this.firstName=e?.firstName,this.lastName=e?.lastName}},te=class{constructor(e){this.customer=e.customer,this.billing=e.billing}},y;(function(e){e.CreditCard=`Credit Card`,e.PayPal=`PayPal`,e.GooglePay=`Google Pay`,e.Venmo=`Venmo`,e.ApplePay=`Apple Pay`})(y||(y={}));var b=class e{get feeAmountCovered(){return this.coverFees?this.fee:0}get fee(){return e.calculateFeeAmount(this.amount)}get total(){return e.calculateTotal(this.amount,this.coverFees)}static calculateTotal(e,t){let n=e+(t?this.calculateFeeAmount(e):0);return isNaN(n)?0:this.roundAmount(n)}static calculateFeeAmount(e){let t=e*.0219+.29;return isNaN(t)?0:this.roundAmount(t)}static roundAmount(e){return Math.round(e*100)/100}constructor(e){this.donationType=e.donationType,this.amount=e.amount,this.coverFees=e.coverFees}},x;(function(e){e.OneTime=`one-time`,e.Monthly=`monthly`,e.Upsell=`up_sell`})(x||(x={}));var ne=class{constructor(e){this.logged_in_user=e?.logged_in_user,this.referrer=e?.referrer,this.fee_amount_covered=e?.fee_amount_covered,this.origin=e?.origin}},re=class{constructor(e){this.customFields=new ne,this.paymentProvider=e.paymentProvider,this.paymentMethodNonce=e.paymentMethodNonce,this.recaptchaToken=e.recaptchaToken,this.customerId=e.customerId,this.deviceData=e.deviceData,this.upsellOnetimeTransactionId=e.upsellOnetimeTransactionId,this.bin=e.bin,this.binName=e.binName,this.amount=e.amount,this.donationType=e.donationType,this.customer=e.customer,this.billing=e.billing,e.customFields&&(this.customFields=e.customFields)}},ie=class{constructor(e){this.id=e.id}},ae=class{constructor(e){this.paymentProvider=e.paymentProvider,this.paymentMethodNonce=e.paymentMethodNonce,this.amount=e.amount,this.donationType=e.donationType,this.transaction_id=e.transaction_id,this.customer_id=e.customer_id,this.customer=new v(e.customer),this.billing=new _(e.billing),e.subscription&&(this.subscription=new ie(e.subscription))}},oe=class{constructor(e){this.code=e.code,this.message=e.message}},se=class{constructor(e){this.message=e.message;let{errors:t=[]}=e;this.errors=t.map(e=>new oe(e))}},ce=class{constructor(e){this.success=e.success,this.success?this.value=new ae(e.value):this.value=new se(e.value)}},S=[5,10,25,50,100,500,1e3],C=new b({donationType:x.OneTime,amount:10,coverFees:!1}),w;(function(e){e.Development=`dev`,e.Production=`prod`})(w||(w={}));function T(e){let t=e.trim();if(!/^[0-9]+([.,][0-9]*)*$/.test(t))return NaN;let n=Math.max(t.lastIndexOf(`,`),t.lastIndexOf(`.`));if(n===-1)return parseFloat(t);let r=t[n],i=r===`,`?`.`:`,`,a=t.length-n-1,o=(e,t)=>RegExp(`^[0-9]{1,3}(\\${t}[0-9]{3})*$`).test(e);if(a<=2){let e=t.slice(0,n);if(!(e.includes(i)?o(e,i):/^[0-9]+$/.test(e)))return NaN;let r=t.slice(n+1);return parseFloat(`${e.split(i).join(``)}.${r||`0`}`)}return a===3&&r===`,`&&o(t,r)?parseFloat(t.split(r).join(``)):NaN}var E=class{constructor(e,t){this.braintreeClient=new m({generator:()=>f(this,void 0,void 0,function*(){return yield this.loadBraintreeScript(`client`),window.braintree.client})}),this.dataCollector=new m({generator:()=>f(this,void 0,void 0,function*(){return yield this.loadBraintreeScript(`data-collector`),window.braintree.dataCollector})}),this.hostedFields=new m({generator:()=>f(this,void 0,void 0,function*(){return yield this.loadBraintreeScript(`hosted-fields`),window.braintree.hostedFields})}),this.venmo=new m({generator:()=>f(this,void 0,void 0,function*(){return yield this.loadBraintreeScript(`venmo`),window.braintree.venmo})}),this.payPal=new m({generator:()=>f(this,void 0,void 0,function*(){return yield this.loadBraintreeScript(`paypal-checkout`),window.braintree.paypalCheckout})}),this.applePay=new m({generator:()=>f(this,void 0,void 0,function*(){return yield this.loadBraintreeScript(`apple-pay`),window.braintree.applePay})}),this.googlePayBraintreeClient=new m({generator:()=>f(this,void 0,void 0,function*(){return yield this.loadBraintreeScript(`google-payment`),window.braintree.googlePayment})}),this.googlePaymentsClient=new m({generator:()=>f(this,void 0,void 0,function*(){return yield this.lazyLoader.loadScript({src:`https://pay.google.com/gp/p/js/pay.js`}),new google.payments.api.PaymentsClient({environment:this.environment===w.Development?`TEST`:`PRODUCTION`})})}),this.recaptchaLibrary=new m({generator:()=>new Promise(e=>{window.donationFormGrecaptchaLoadedCallback=()=>{setTimeout(()=>{delete window.donationFormGrecaptchaLoadedCallback},10),e(window.grecaptcha)},this.lazyLoader.loadScript({src:`https://www.google.com/recaptcha/api.js?onload=donationFormGrecaptchaLoadedCallback&render=explicit`})})}),this.paypalLibrary=new m({generator:()=>f(this,void 0,void 0,function*(){return yield this.lazyLoader.loadScript({src:`https://www.paypalobjects.com/api/checkout.js`,attributes:{"data-version-4":``,"log-level":`warn`}}),window.paypal})}),this.braintreeVersion=`3.62.2`,this.environment=w.Development,this.lazyLoader=e,this.environment=t}loadBraintreeScript(e){return f(this,void 0,void 0,function*(){let t=`${e}.${this.environment===w.Production?`min.js`:`js`}`,n=`https://js.braintreegateway.com/web/${this.braintreeVersion}/js/${t}`;yield this.lazyLoader.loadScript({src:n})})}},le=class extends Error{constructor(e){super(e),this.name=`DonationFormError`}},ue=class{on(e,t){return this.emitter.on(e,t)}constructor(e){this.instance=new m({generator:()=>f(this,void 0,void 0,function*(){let e=yield this.braintreeManager.instance.get();return yield this.createHostedFields(e)})}),this.emitter=h(),this.braintreeManager=e.braintreeManager,this.hostedFieldClient=e.hostedFieldClient,this.hostedFieldConfig=e.hostedFieldConfig,this.maxRetryCount=e.maxRetryCount??2,this.loadTimeout=(e.loadTimeout??6)*1e3}createHostedFields(e){return f(this,arguments,void 0,function*(e,t=0){this.hostedFieldConfig.hostedFieldContainer.resetHostedFields();try{let t,n=new Promise((e,n)=>{t=window.setTimeout(()=>{n(new le(`Timeout loading Hosted Fields`))},this.loadTimeout)}),r=new Promise((n,r)=>f(this,void 0,void 0,function*(){try{let r=yield this.hostedFieldClient.create({client:e,styles:this.hostedFieldConfig.hostedFieldStyle,fields:this.hostedFieldConfig.hostedFieldFieldOptions});window.clearTimeout(t),n(r)}catch(e){e instanceof Error&&e.message.includes(`Hosted Fields timed out`)||r(e)}}));return yield Promise.race([n,r])}catch(n){if(t>=this.maxRetryCount)throw this.emitter.emit(`hostedFieldsFailed`,n),n;let r=t+1;return this.emitter.emit(`hostedFieldsRetry`,r),this.createHostedFields(e,r)}})}tokenizeHostedFields(){return f(this,void 0,void 0,function*(){return(yield this.instance.get())?.tokenize()})}focusField(e){return f(this,void 0,void 0,function*(){(yield this.instance.get())?.focus(e)})}markFieldErrors(e){this.hostedFieldConfig.hostedFieldContainer.markFieldErrors(e)}removeFieldErrors(e){this.hostedFieldConfig.hostedFieldContainer.removeFieldErrors(e)}showErrorMessage(e){this.hostedFieldConfig.hostedFieldContainer.showErrorMessage(e)}hideErrorMessage(){this.hostedFieldConfig.hostedFieldContainer.hideErrorMessage()}},de=class{constructor(e){this.session=e.session,this.donationInfo=e.donationInfo,this.applePayInstance=e.applePayInstance,this.braintreeManager=e.braintreeManager}onvalidatemerchant(e){return f(this,void 0,void 0,function*(){return new Promise((t,n)=>{this.applePayInstance.performValidation({validationURL:e.validationURL,displayName:`Internet Archive`},(e,r)=>{var i;e?((i=this.delegate)==null||i.paymentFailed(e),this.session.abort(),n(`Merchant validation error: ${e}`)):(this.session.completeMerchantValidation(r),t())})})})}oncancel(){return f(this,void 0,void 0,function*(){var e;(e=this.delegate)==null||e.paymentCancelled()})}onpaymentauthorized(e){return f(this,void 0,void 0,function*(){var t,n,r,i;let a;try{a=yield this.applePayInstance.tokenize({token:e.payment.token})}catch(e){(t=this.delegate)==null||t.paymentFailed(e),this.session.completePayment(ApplePaySession.STATUS_FAILURE);return}let o=e.payment,s=o.billingContact,c=o.shippingContact,l=s?.addressLines,u,d;l&&(u=l[0],d=l[1]);let f=new _({streetAddress:u,extendedAddress:d,locality:s?.locality,region:s?.administrativeArea,postalCode:s?.postalCode,countryCodeAlpha2:s?.countryCode}),p=new v({email:c?.emailAddress,firstName:c?.givenName,lastName:c?.familyName});try{let e=yield this.braintreeManager.submitDonation({nonce:a.nonce,paymentProvider:y.ApplePay,donationInfo:this.donationInfo,billingInfo:f,customerInfo:p});e.success?((n=this.delegate)==null||n.paymentComplete(e),this.session.completePayment(ApplePaySession.STATUS_SUCCESS)):((r=this.delegate)==null||r.paymentFailed(`Failure submitting data`),this.session.completePayment(ApplePaySession.STATUS_FAILURE))}catch(e){(i=this.delegate)==null||i.paymentFailed(e),this.session.completePayment(ApplePaySession.STATUS_FAILURE)}})}},fe=class{constructor(e){this.braintreeManager=e.braintreeManager,this.applePayClient=e.applePayClient,this.applePaySessionManager=e.applePaySessionManager,this.instance=e.instancePromisedSingleton??new m({generator:()=>f(this,void 0,void 0,function*(){if(!this.applePaySessionManager.canMakePayments())return;let e=yield this.braintreeManager.instance.get();return yield this.applePayClient.create({client:e})})})}isAvailable(){return f(this,void 0,void 0,function*(){try{return!!(yield this.instance.get())}catch{return!1}})}createPaymentRequest(e,t){return f(this,void 0,void 0,function*(){let e=yield this.instance.get(),n=`Internet Archive Monthly`;t.donationType===x.OneTime&&(n=`Internet Archive`);let r=e.createPaymentRequest({total:{label:n,amount:t.total},requiredBillingContactFields:[`postalAddress`],requiredShippingContactFields:[`name`,`email`]}),i=this.applePaySessionManager.createNewPaymentSession(r),a=new de({donationInfo:t,session:i,applePayInstance:e,braintreeManager:this.braintreeManager});return i.onvalidatemerchant=a.onvalidatemerchant.bind(a),i.onpaymentauthorized=a.onpaymentauthorized.bind(a),i.oncancel=a.oncancel.bind(a),i.begin(),a})}},pe=class{constructor(e){this.braintreeManager=e.braintreeManager,this.venmoClient=e.venmoClient,this.venmoProfileId=e.venmoProfileId,this.instance=e.instancePromisedSingleton??new m({generator:()=>this.braintreeManager.instance.get().then(e=>this.venmoClient.create({client:e,profileId:this.venmoProfileId}))})}isBrowserSupported(){return f(this,void 0,void 0,function*(){return this.isMobileFirefox()?!1:(yield this.instance.get())?.isBrowserSupported()??!1})}startPayment(){return f(this,void 0,void 0,function*(){return(yield this.instance.get())?.tokenize()})}isMobileFirefox(){let e=navigator.userAgent.indexOf(`FxiOS`)!==-1,t=navigator.userAgent.indexOf(`Firefox`)!==-1,n=navigator.userAgent.indexOf(`Mobile`)!==-1;return(t||e)&&n}},me=t(((e,t)=>{(function(n,r){typeof e==`object`&&t!==void 0?t.exports=r():typeof define==`function`&&define.amd?define(r):(n=n||self,n.currency=r())})(e,function(){function e(a,o){if(!(this instanceof e))return new e(a,o);o=Object.assign({},n,o);var s=10**o.precision;this.intValue=a=t(a,o),this.value=a/s,o.increment=o.increment||1/s,o.groups=o.useVedic?i:r,this.s=o,this.p=s}function t(t,n){var r=2<arguments.length&&arguments[2]!==void 0?arguments[2]:!0,i=n.decimal,a=n.errorOnInvalid,o=n.fromCents,s=10**n.precision,c=t instanceof e;if(c&&o)return t.intValue;if(typeof t==`number`||c)i=c?t.value:t;else if(typeof t==`string`)a=RegExp(`[^-\\d`+i+`]`,`g`),i=RegExp(`\\`+i,`g`),i=(i=t.replace(/\((.*)\)/,`-$1`).replace(a,``).replace(i,`.`))||0;else{if(a)throw Error(`Invalid Input`);i=0}return o||(i=(i*s).toFixed(4)),r?Math.round(i):i}var n={symbol:`$`,separator:`,`,decimal:`.`,errorOnInvalid:!1,precision:2,pattern:`!#`,negativePattern:`-!#`,format:function(e,t){var n=t.pattern,r=t.negativePattern,i=t.symbol,a=t.separator,o=t.decimal;t=t.groups;var s=(``+e).replace(/^-/,``).split(`.`),c=s[0];return s=s[1],(0<=e.value?n:r).replace(`!`,i).replace(`#`,c.replace(t,`$1`+a)+(s?o+s:``))},fromCents:!1},r=/(\d)(?=(\d{3})+\b)/g,i=/(\d)(?=(\d\d)+\d\b)/g;return e.prototype={add:function(n){var r=this.s,i=this.p;return e((this.intValue+t(n,r))/(r.fromCents?1:i),r)},subtract:function(n){var r=this.s,i=this.p;return e((this.intValue-t(n,r))/(r.fromCents?1:i),r)},multiply:function(t){var n=this.s;return e(this.intValue*t/(n.fromCents?1:10**n.precision),n)},divide:function(n){var r=this.s;return e(this.intValue/t(n,r,!1),r)},distribute:function(t){var n=this.intValue,r=this.p,i=this.s,a=[],o=Math[0<=n?`floor`:`ceil`](n/t),s=Math.abs(n-o*t);for(r=i.fromCents?1:r;t!==0;t--){var c=e(o/r,i);0<s--&&(c=c[0<=n?`add`:`subtract`](1/r)),a.push(c)}return a},dollars:function(){return~~this.value},cents:function(){return~~(this.intValue%this.p)},format:function(e){var t=this.s;return typeof e==`function`?e(this,t):t.format(this,Object.assign({},t,e))},toString:function(){var e=this.s,t=e.increment;return(Math.round(this.intValue/this.p/t)*t).toFixed(e.precision)},toJSON:function(){return this.value}},e})})),D=e(me()),he=class{constructor(e){this.donationInfo=e.donationInfo,this.paypalInstance=e.paypalInstance}payment(){return f(this,void 0,void 0,function*(){var e;let t=this.donationInfo.donationType===x.OneTime?`checkout`:`vault`,n={flow:t,intent:`capture`};return n.enableShippingAddress=!0,t===`checkout`?(n.amount=this.donationInfo.total,n.currency=`USD`):n.billingAgreementDescription=`Subscribe to donate ${(0,D.default)(this.donationInfo.total,{symbol:`$`}).format()} monthly`,(e=this.delegate)==null||e.payPalPaymentStarted(this,n),this.paypalInstance.createPayment(n)})}onAuthorize(e){return f(this,void 0,void 0,function*(){var t;let n=yield this.paypalInstance.tokenizePayment(e);return(t=this.delegate)==null||t.payPalPaymentAuthorized(this,n),n})}onConfirm(e){return f(this,void 0,void 0,function*(){var t;let n=yield this.paypalInstance.tokenizePayment(e);return(t=this.delegate)==null||t.payPalPaymentConfirmed(this,n),n})}onCancel(e){var t;(t=this.delegate)==null||t.payPalPaymentCancelled(this,e)}onError(e){var t;console.error(`PayPal error`,e),(t=this.delegate)==null||t.payPalPaymentError(this,e)}},ge=class{constructor(e){this.braintreeManager=e.braintreeManager,this.paypalClient=e.paypalClient,this.paypalButtonGenerator=e.paypalButton,this.hostingEnvironment=e.hostingEnvironment,this.instance=new m({generator:()=>this.braintreeManager.instance.get().then(e=>this.paypalClient.create({client:e}))})}renderPayPalButton(e){return f(this,void 0,void 0,function*(){let t=this.hostingEnvironment===w.Development?`sandbox`:`production`,n=yield this.instance.get();if(!n)return;let r=new he({donationInfo:e.donationInfo,paypalInstance:n});return this.paypalButtonGenerator.render({env:t,style:e.style,payment:r.payment.bind(r),onAuthorize:r.onAuthorize.bind(r),onCancel:r.onCancel.bind(r),onError:r.onError.bind(r),funding:{disallowed:[paypal.FUNDING.VENMO]}},e.selector),r})}},_e=class e{canMakePayments(){return`ApplePaySession`in window&&ApplePaySession.supportsVersion(e.VERSION)&&ApplePaySession.canMakePayments()}createNewPaymentSession(t){return new ApplePaySession(e.VERSION,t)}};_e.VERSION=3;var ve=class{isBrowserSupported(){return f(this,void 0,void 0,function*(){return this.paymentsClient.isReadyToPay({apiVersion:2,apiVersionMinor:0,allowedPaymentMethods:[{type:`CARD`,parameters:{allowedAuthMethods:[`PAN_ONLY`],allowedCardNetworks:[`AMEX`,`DISCOVER`,`INTERAC`,`JCB`,`MASTERCARD`,`VISA`]}}],existingPaymentMethodRequired:!1}).then(e=>e.result)})}constructor(e){this.braintreeManager=e.braintreeManager,this.googlePayMerchantId=e.googlePayMerchantId,this.googlePayBraintreeClient=e.googlePayBraintreeClient,this.paymentsClient=e.googlePaymentsClient,this.instance=new m({generator:()=>this.braintreeManager.instance.get().then(e=>this.googlePayBraintreeClient.create({client:e,googlePayVersion:2,googleMerchantId:this.googlePayMerchantId}))})}},ye=class{on(e,t){return this.emitter.on(e,t)}constructor(e){this.creditCardHandler=new m({generator:()=>f(this,void 0,void 0,function*(){let e=yield this.paymentClients.hostedFields.get(),t=new ue({braintreeManager:this.braintreeManager,hostedFieldClient:e,hostedFieldConfig:this.hostedFieldConfig});return t.on(`hostedFieldsRetry`,e=>{this.emitter.emit(`hostedFieldsRetry`,e)}),t.on(`hostedFieldsFailed`,e=>{this.emitter.emit(`hostedFieldsFailed`,e)}),t})}),this.applePayHandler=new m({generator:()=>f(this,void 0,void 0,function*(){let e=yield this.paymentClients.applePay.get(),t=new _e;return new fe({braintreeManager:this.braintreeManager,applePayClient:e,applePaySessionManager:t})})}),this.venmoHandler=new m({generator:()=>f(this,void 0,void 0,function*(){let e=yield this.paymentClients.venmo.get();if(this.venmoProfileId)return new pe({braintreeManager:this.braintreeManager,venmoClient:e,venmoProfileId:this.venmoProfileId})})}),this.paypalHandler=new m({generator:()=>f(this,void 0,void 0,function*(){let e=this.paymentClients.paypalLibrary.get(),t=this.paymentClients.payPal.get(),n=yield Promise.all([t,e]);return new ge({braintreeManager:this.braintreeManager,paypalClient:n[0],paypalButton:n[1].Button,hostingEnvironment:this.hostingEnvironment})})}),this.googlePayHandler=new m({generator:()=>f(this,void 0,void 0,function*(){let e=this.paymentClients.googlePaymentsClient.get(),t=this.paymentClients.googlePayBraintreeClient.get(),n=yield Promise.all([t,e]);return new ve({braintreeManager:this.braintreeManager,googlePayMerchantId:this.googlePayMerchantId,googlePayBraintreeClient:n[0],googlePaymentsClient:n[1]})})}),this.hostingEnvironment=w.Development,this.emitter=h(),this.braintreeManager=e.braintreeManager,this.venmoProfileId=e.venmoProfileId,this.googlePayMerchantId=e.googlePayMerchantId,this.paymentClients=e.paymentClients,this.hostingEnvironment=e.hostingEnvironment,this.hostedFieldConfig=e.hostedFieldConfig}},be=class{on(e,t){return this.emitter.on(e,t)}startup(){return f(this,void 0,void 0,function*(){return this.collectDeviceData()})}submitDonation(e){return f(this,void 0,void 0,function*(){let t=new ne;t.fee_amount_covered=e.donationInfo.feeAmountCovered,t.logged_in_user=this.loggedInUser,t.referrer=this.referrer,t.origin=this.origin;let n=b.calculateTotal(e.donationInfo.amount,e.donationInfo.coverFees),r=new re({deviceData:this.deviceData,paymentProvider:e.paymentProvider,paymentMethodNonce:e.nonce,amount:n,donationType:e.donationInfo.donationType,customer:e.customerInfo,billing:e.billingInfo,customFields:t,upsellOnetimeTransactionId:e.upsellOnetimeTransactionId,customerId:e.customerId,recaptchaToken:e.recaptchaToken,bin:e.bin,binName:e.binName});return new ce(yield this.endpointManager.submitData(r))})}submitUpsellDonation(e){return f(this,void 0,void 0,function*(){let t=e.oneTimeDonationResponse,n=new b({amount:e.amount,donationType:x.Upsell,coverFees:!1});return this.submitDonation({nonce:t.paymentMethodNonce,paymentProvider:t.paymentProvider,customerId:t.customer_id,donationInfo:n,customerInfo:t.customer,billingInfo:t.billing,upsellOnetimeTransactionId:t.transaction_id})})}donationSuccessful(e){this.endpointManager.donationSuccessful(e)}collectDeviceData(){return f(this,void 0,void 0,function*(){if(this.deviceDataCollectionStarted)return;this.deviceDataCollectionStarted=!0;let e=yield this.instance.get();if(e)return this.paymentClients.dataCollector.get().then(t=>t?.create({client:e,kount:!1,paypal:!0})).then(e=>{this.deviceData=e?.deviceData})})}constructor(e){this.emitter=h(),this.instance=new m({generator:()=>f(this,void 0,void 0,function*(){return(yield this.paymentClients.braintreeClient.get())?.create({authorization:this.authorizationToken})})}),this.deviceDataCollectionStarted=!1,this.hostingEnvironment=w.Development,this.authorizationToken=e.authorizationToken,this.endpointManager=e.endpointManager,this.hostingEnvironment=e.hostingEnvironment,this.paymentClients=e.paymentClients,this.referrer=e.referrer,this.loggedInUser=e.loggedInUser,this.origin=e.origin,this.paymentProviders=new ye({braintreeManager:this,paymentClients:this.paymentClients,venmoProfileId:e.venmoProfileId,googlePayMerchantId:e.googlePayMerchantId,hostingEnvironment:e.hostingEnvironment,hostedFieldConfig:e.hostedFieldConfig}),this.paymentProviders.on(`hostedFieldsRetry`,e=>{this.emitter.emit(`paymentProvidersHostedFieldsRetry`,e)}),this.paymentProviders.on(`hostedFieldsFailed`,e=>{this.emitter.emit(`paymentProvidersHostedFieldsFailed`,e)})}setReferrer(e){this.referrer=e}setLoggedInUser(e){this.loggedInUser=e}setOrigin(e){this.origin=e}},O;(function(e){e.Number=`number`,e.CVV=`cvv`,e.ExpirationDate=`expirationDate`})(O||(O={}));var xe=class{fieldFor(e){switch(e){case O.Number:return this.number;case O.CVV:return this.cvv;case O.ExpirationDate:return this.expirationDate}}markFieldErrors(e){e.forEach(e=>{let t=this.fieldFor(e);t.parentElement.error=!0})}removeFieldErrors(e){e.forEach(e=>{let t=this.fieldFor(e);t.parentElement.error=!1})}showErrorMessage(e){let t=e??`Some payment information below is missing or incorrect.`;this.errorContainer.innerHTML=t,this.errorContainer.style.display=`block`}hideErrorMessage(){this.errorContainer.style.display=`none`}resetHostedFields(){[this.number,this.cvv,this.expirationDate].forEach(e=>{for(;e.firstChild;)e.firstChild.remove()})}constructor(e){this.number=e.number,this.cvv=e.cvv,this.expirationDate=e.expirationDate,this.errorContainer=e.errorContainer}},Se=class{constructor(e){this.started=!1,this.braintreeManager=e.braintreeManager,this.donationFlowModalManager=e.donationFlowModalManager,this.recaptchaManager=e.recaptchaManager,this.emitter=h()}on(e,t){return this.emitter.on(e,t)}startup(){return f(this,void 0,void 0,function*(){if(this.started)return;this.started=!0;let e=yield this.braintreeManager?.paymentProviders.creditCardHandler.get(),t=yield e?.instance.get();t?.on(`focus`,t=>{let{emittedBy:n,fields:r}=t,{container:i}=r[n];i.parentElement.error=!1,e.hideErrorMessage()}),t?.on(`blur`,e=>{let{emittedBy:t,fields:n}=e,{container:r,isEmpty:i,isValid:a}=n[t];!i&&!a&&(r.parentElement.error=!0)}),t?.on(`validityChange`,e=>{let{fields:t}=e,n=t.cvv.isValid&&t.expirationDate.isValid&&t.number.isValid;this.emitter.emit(`validityChanged`,n)})})}tokenizeFields(){return f(this,void 0,void 0,function*(){let e,t=yield this.braintreeManager.paymentProviders.creditCardHandler.get();try{e=yield t?.tokenizeHostedFields()}catch(e){this.handleHostedFieldTokenizationError(e);return}return e})}paymentInitiated(e,t,n){return f(this,void 0,void 0,function*(){let r;try{r=yield this.recaptchaManager.execute()}catch{this.donationFlowModalManager.showErrorModal({message:`Recaptcha failure`});return}this.donationFlowModalManager.startDonationSubmissionFlow({nonce:e.nonce,paymentProvider:y.CreditCard,recaptchaToken:r,bin:e.details.bin,donationInfo:t,customerInfo:n.customer,billingInfo:n.billing})})}handleHostedFieldTokenizationError(e){return f(this,void 0,void 0,function*(){let t=yield this.braintreeManager.paymentProviders.creditCardHandler.get();switch(t.showErrorMessage(),e.code){case`HOSTED_FIELDS_FIELDS_EMPTY`:t.markFieldErrors([O.Number,O.CVV,O.ExpirationDate]);break;case`HOSTED_FIELDS_FIELDS_INVALID`:Object.keys(e.details.invalidFields).forEach(e=>{t.markFieldErrors([e])});break;case`HOSTED_FIELDS_TOKENIZATION_FAIL_ON_DUPLICATE`:break;case`HOSTED_FIELDS_TOKENIZATION_CVV_VERIFICATION_FAILED`:t.markFieldErrors([O.CVV]);break;case`HOSTED_FIELDS_FAILED_TOKENIZATION`:break;case`HOSTED_FIELDS_TOKENIZATION_NETWORK_ERROR`:break;default:break}})}},Ce=class{keydown(e){let t=e.key;if(e.metaKey)return;switch(t){case`Tab`:case`Delete`:case`Backspace`:case`ArrowLeft`:case`ArrowRight`:case`ArrowUp`:case`ArrowDown`:return}let n=e.target,r=n.value;`${r.slice(0,n.selectionStart??0)}${t}${r.slice(n.selectionEnd??0)}`.match(/^[0-9]+(\.[0-9]{0,2})?$/g)||e.preventDefault()}},k;(function(e){e.YesButton=`YesButton`,e.PayPalUpsellSlot=`PayPalUpsellSlot`})(k||(k={}));var A=class extends c{constructor(){super(...arguments),this.yesButtonMode=k.YesButton,this.amount=5,this.currencyValidator=new Ce}render(){return a`
      <h3>Thank you for donating!</h3>
      <button @click=${this.noThanksSelected} class="cta-button" id="no-button">Continue</button>
      <p class="or_separator"><span>or</span></p>
      <h3>Join our Monthly Giving Circle</h3>
      <p class="appeal">Monthly support helps us reliably plan for the future.</p>
      <div class="monthly-amount">
        <h1>Enter your monthly amount</h1>
        <div class="amount-input">
          <span class="dollar-symbol">$</span>
          <input
            id="amount-input"
            type="text"
            tabindex="0"
            value=${this.amount}
            @input=${this.amountChanged}
            @keydown=${this.currencyValidator.keydown}
          />
        </div>
        <div class="error ${this.error?``:`hidden`}">${this.error}</div>
      </div>

      ${this.yesButton}
    `}get yesButton(){switch(this.yesButtonMode){case k.YesButton:return a`
          <button
            class="cta-button"
            tabindex="0"
            id="yes-button"
            @click=${this.yesSelected}
            .disabled=${this.error!==void 0}
          >
            YES, I'll donate monthly
          </button>
        `;case k.PayPalUpsellSlot:return a`
          <div class="paypal-upsell-slot-container">
            <div class="paypal-upsell-slot-blocker ${this.error?``:`hidden`}"></div>
            <button class="cta-button" id="paypal-cover-button">YES, I'll donate monthly</button>
            <slot class="paypal-upsell-slot"></slot>
          </div>
        `}}amountChanged(e){let t=e.target.value;t.length!==0&&this.handleCustomAmountInput(t)}handleCustomAmountInput(e){let t=T(e);isNaN(t)?this.error=a` Please enter a valid amount. `:this.processAmount(t)}processAmount(e){if(e>=1e4){this.error=a`
        To make a donation of $10,000 or more, please contact our philanthropy department at
        <a href="mailto:donations@archive.org">donations@archive.org</a>
      `;return}if(e<1){this.amountInput&&this.amountInput.value.length>0&&(this.error=a` The minimum donation amount is $1. `);return}this.error=void 0,this.amount=e;let t=new CustomEvent(`amountChanged`,{detail:{amount:this.amount}});this.dispatchEvent(t)}yesSelected(){let e=new CustomEvent(`yesSelected`,{detail:{amount:this.amount}});this.dispatchEvent(e)}noThanksSelected(){this.dispatchEvent(new Event(`noThanksSelected`))}static get styles(){let e=n`var(--upsellCTAButtonColor, #194880)`,t=n`var(--upsellCTAButtonDisabledColor, rgba(109,148,201,0.5))`;return n`
      .monthly-amount {
        background-color: #fff;
        padding: 0.5rem 0.625rem;
        border-radius: 5px;
        text-align: center;
        margin-bottom: 0.5rem;
        margin-top: 0;
      }

      .monthly-amount h1 {
        font-size: 1.8rem;
        font-weight: bold;
        text-align: center;
        line-height: 1.2em;
        margin: 0;
        padding: 0.5rem 0 0 0;
      }

      .hidden {
        display: none;
      }

      h3 {
        text-align: center;
        font-size: 1.8rem;
        margin: 0 1rem 0.5rem 1rem;
      }

      .appeal {
        text-align: center;
        font-size: 1.6rem;
        margin: 0.5rem 1rem;
      }

      .amount-input {
        transform: translate(${n`var(--upsellAmountInputOffset, -1rem)`}, 0); /* translate slightly to center the input */
      }

      .amount-input .dollar-symbol {
        font-size: 1.8rem;
        font-weight: bold;
      }

      .amount-input input {
        width: 100px;
        text-align: center;
        border: none;
        border-bottom: 1px solid gray;
        font-weight: bold;
        font-size: 3.4rem;
      }

      .cta-button {
        font-size: 2rem;
        display: block;
        width: 100%;
        margin-top: 0.5rem;
        padding: 1rem 2rem;
        background-color: ${e};
        color: #fff;
        border-radius: 5px;
        border: 0;
        font-weight: bold;
        line-height: normal;
        outline: none;
        cursor: pointer;
      }

      .cta-button:disabled {
        background-color: ${t};
        cursor: not-allowed;
      }

      .paypal-upsell-slot {
        text-align: center;
      }

      .paypal-upsell-slot-blocker {
        position: absolute;
        width: 100%;
        height: 4.5rem;
        bottom: 0;
        z-index: 250;
        cursor: not-allowed;
        background-color: rgba(255, 255, 255, 0.5);
      }

      .paypal-upsell-slot-blocker.hidden {
        display: none;
      }

      #paypal-cover-button {
        position: absolute;
        width: 100%;
        bottom: 0;
      }

      .paypal-upsell-slot-container {
        position: relative;
      }

      .paypal-upsell-slot-container .paypal-cta {
        font-size: 2.4rem;
        font-weight: bold;
        margin: 0 1rem 1rem 1rem;
        text-align: center;
      }

      .error {
        font-size: 1.4rem;
        margin: 0.5rem 0;
        color: red;
      }

      .or_separator {
        position: relative;
        margin: 0 2rem;
        font-size: 2.6rem;
        font-weight: bold;
        text-transform: uppercase;
        text-align: center;
      }

      .or_separator:before {
        position: absolute;
        top: calc(50% - 1px);
        right: 0;
        left: 0;
        height: 2px;
        content: '';
        background: #333;
      }

      .or_separator span {
        display: inline-block;
        position: relative;
        padding: 0 1rem;
        background: #f5f5f7;
      }
    `}};p([r({type:String})],A.prototype,`yesButtonMode`,void 0),p([r({type:Number})],A.prototype,`amount`,void 0),p([r({type:Object})],A.prototype,`error`,void 0),p([d(`#amount-input`)],A.prototype,`amountInput`,void 0),A=p([u(`upsell-modal-content`)],A);var j=class extends c{constructor(){super(...arguments),this.amount=5,this.currencyType=`$`,this.donationType=x.OneTime,this.confirmDonation=()=>{},this.cancelDonation=()=>{}}get confirmationText(){let e=(0,D.default)(this.amount,{symbol:this.currencySymbol}).format();return a`
      <p>
        You are about to make a <b>${this.donationType}</b> donation of
        <b>${e} ${this.currencyType}</b> to the Internet Archive.
      </p>
    `}get confirmUpsellText(){return a`<p>
      You are about to begin making <b>monthly</b> donations of
      <b>${(0,D.default)(this.amount,{symbol:this.currencySymbol}).format()} ${this.currencyType}</b> to the Internet Archive. (Your first recurring
      contribution will be next month.)
    </p>`}confirm(){this===null||this===void 0||this.confirmDonation()}cancel(){this===null||this===void 0||this.cancelDonation()}get confirmCTA(){return this.donationType===x.Upsell?`Start monthly donation`:`Complete donation`}render(){return a`
      ${this.donationType===x.Upsell?this.confirmUpsellText:this.confirmationText}

      <div class="cta-group">
        <button id="confirm" @click=${()=>this.confirm()}>${this.confirmCTA}</button>
        <button id="cancel" @click=${()=>this.cancel()}>Cancel</button>
      </div>
    `}static get styles(){return n`
      :host {
        display: block;
      }

      button {
        outline: none;
        cursor: pointer;
      }

      button#confirm {
        font-size: 2rem;
        display: block;
        width: 100%;
        margin-top: 0.5rem;
        padding: 1rem 2rem;
        background-color: ${n`var(--upsellCTAButtonColor, #194880)`};
        color: #fff;
        border-radius: 5px;
        border: 0;
        font-weight: bold;
        line-height: normal;
      }

      button#cancel {
        margin-top: 1rem;
        border: 0;
        text-decoration: underline;
        background-color: transparent;
      }

      button:disabled {
        background-color: ${n`var(--upsellCTAButtonDisabledColor, rgba(109,148,201,0.5))`};
        cursor: not-allowed;
      }
    }`}get currencySymbol(){switch(this.currencyType){case`AUD`:return`AU$`;case`BRL`:return`R$`;case`CAD`:return`CA$`;case`CHF`:return`Fr`;case`CNY`:return`¥`;case`CZK`:return`Kč`;case`DKK`:return`Kr`;case`EUR`:return`€`;case`GBP`:return`£`;case`HKD`:return`HK$`;case`HUF`:return`Ft`;case`ILS`:return`₪`;case`JPY`:return`¥`;case`MXN`:return`MX$`;case`MYR`:return`RM`;case`NOK`:return`kr`;case`PLN`:return`zł`;case`RUB`:return`₽`;case`SEK`:return`kr`;case`SGD`:return`S$`;case`THB`:return`฿`;case`TYD`:return`NT$`;default:return`$`}}};p([r({type:Number})],j.prototype,`amount`,void 0),p([r({type:String})],j.prototype,`currencyType`,void 0),p([r({type:String})],j.prototype,`donationType`,void 0),p([r({type:Function})],j.prototype,`confirmDonation`,void 0),p([r({type:Function})],j.prototype,`cancelDonation`,void 0),j=p([u(`confirm-donation-modal`)],j);var we=class extends c{render(){return a`
      <div class="container">
        <a
          href="https://help.archive.org/help/why-is-there-a-problem-processing-my-donation/"
          rel="noopener"
          target="_blank"
        >
          Questions?
        </a>
      </div>
    `}static get styles(){return n`
      .container {
        margin-top: ${n`var(--errorModalQuestionsLinkTopMargin, 1rem)`};
        text-align: center;
      }

      a,
      a:link,
      a:visited {
        color: ${n`var(--errorModalQuestionsLinkFontColor, #333)`};
        font-size: ${n`var(--errorModalQuestionsLinkFontSize, 1.4rem)`};
      }
    `}};we=p([u(`donation-form-error-modal-content`)],we);var M;(function(e){e.Blue=`#497fbf`,e.Green=`#55A183`,e.Red=`#691916`})(M||(M={}));var Te=class e{constructor(e){this.modalManager=e.modalManager,this.braintreeManager=e.braintreeManager,this.analytics=e.analytics}closeModal(){this.modalManager.closeModal()}showProcessingModal(){let e=new g({headerColor:M.Blue,showProcessingIndicator:!0,closeOnBackdropClick:!1,showCloseButton:!1,processingImageMode:`processing`,title:a` Processing... `});this.modalManager.showModal({config:e})}showThankYouModal(e){let t=new g({showProcessingIndicator:!0,processingImageMode:`complete`,headerColor:M.Green,title:a` Thank You! `});this.modalManager.showModal({config:t});let n=`Donated-${e.successResponse.paymentProvider.replace(/\s+/g,``)}`;e.upsellSuccessResponse&&(n+=`-upsell`);let r=e.successResponse.donationType;this.analytics.logDonationFlowEvent(n,r),this.braintreeManager.donationSuccessful(e)}showErrorModal(e){let t=new g({headerColor:M.Red,title:a` Processing error `,headline:a` There's been a problem completing your donation. `,message:a` ${e?.message} `});this.modalManager.showModal({config:t,userClosedModalCallback:e?.userClosedModalCallback,customModalContent:a`
        <donation-form-error-modal-content></donation-form-error-modal-content>
      `})}showConfirmationStepModal(e){let t=()=>{e?.confirmDonationCB()},n=()=>{e?.cancelDonationCB()},r=e.donationType===x.Upsell?`Confirm monthly donation`:`Complete donation`,i=new g({closeOnBackdropClick:!1,headerColor:M.Green,title:a`${r}`,message:a`
        <confirm-donation-modal
          .amount="${e.amount}"
          .currencyType="${e.currencyType}"
          .donationType="${e.donationType}"
          .confirmDonation=${t}
          .cancelDonation=${n}
        ></confirm-donation-modal>
      `});return this.modalManager.showModal({config:i,userClosedModalCallback:n})}showUpsellModal(t){let n=new g({headerColor:M.Green,title:a` Donation received `,processingImageMode:`complete`,showProcessingIndicator:!0}),r=e.getDefaultUpsellAmount(t.oneTimeAmount);t.amountChanged&&t.amountChanged(r);let i=a`
      <upsell-modal-content
        .amount=${r}
        .yesButtonMode=${t?.ctaMode??k.YesButton}
        @yesSelected=${e=>t?.yesSelected?t.yesSelected(e.detail.amount):void 0}
        @noThanksSelected=${t?.noSelected}
        @amountChanged=${e=>t?.amountChanged?t.amountChanged(e.detail.amount):void 0}
      >
        <slot name="paypal-upsell-button"></slot>
      </upsell-modal-content>
    `;return this.modalManager.showModal({config:n,customModalContent:i,userClosedModalCallback:t?.userClosedModalCallback})}startDonationSubmissionFlow(e){return f(this,void 0,void 0,function*(){this.showProcessingModal();try{let t=yield this.braintreeManager.submitDonation(e);if(t.success)return this.handleSuccessfulDonationResponse(e.donationInfo,t.value),t;{let e=t.value;return this.showErrorModal({message:e.message}),t}}catch(e){this.showErrorModal({message:`${e}`}),console.error(`error getting a response`,e);return}})}upsellModalYesSelected(e,t){return f(this,void 0,void 0,function*(){this.showProcessingModal();try{let n=yield this.braintreeManager.submitUpsellDonation({oneTimeDonationResponse:e,amount:t});if(n.success)this.completeUpsell({successResponse:e,upsellSuccessResponse:n.value});else{let e=n.value;this.showErrorModal({message:e.message})}return n}catch(e){this.showErrorModal({message:`${e}`}),console.error(`error getting a response`,e);return}})}completeUpsell(e){this.showThankYouModal(e);let t=`Donated-${e.successResponse.paymentProvider.replace(/\s+/g,``)}-upsell`,n=e.successResponse.donationType;this.analytics.logDonationFlowEvent(t,n),this.braintreeManager.donationSuccessful(e)}static getDefaultUpsellAmount(e){let t=5;return e<=10?t=8:e>10&&e<=25?t=10:e>25&&e<=100?t=25:e>100&&(t=50),t}handleSuccessfulDonationResponse(e,t){switch(e.donationType){case x.OneTime:this.showUpsellModal({oneTimeAmount:t.amount,yesSelected:e=>{this.upsellModalYesSelected(t,e)},noSelected:()=>{this.showThankYouModal({successResponse:t})},userClosedModalCallback:()=>{this.showThankYouModal({successResponse:t})}});break;case x.Monthly:this.showThankYouModal({successResponse:t});break;case x.Upsell:break;default:break}}},Ee=class{constructor(e){this.upsellButtonDataSource=e.upsellButtonDataSource,this.oneTimePayload=e.oneTimePayload,this.oneTimeSuccessResponse=e.oneTimeSuccessResponse}},De=class{updateDonationInfo(e){this.buttonDataSource&&(this.buttonDataSource.donationInfo=e)}updateUpsellDonationInfo(e){this.upsellButtonDataSourceContainer&&(this.upsellButtonDataSourceContainer.upsellButtonDataSource.donationInfo=e)}constructor(e){this.emitter=h(),this.braintreeManager=e.braintreeManager,this.donationFlowModalManager=e.donationFlowModalManager}on(e,t){return this.emitter.on(e,t)}payPalPaymentStarted(e,t){return f(this,void 0,void 0,function*(){this.emitter.emit(`payPalPaymentStarted`,e,t)})}payPalPaymentAuthorized(e,t){return f(this,void 0,void 0,function*(){let{donationType:n,total:r}=e.donationInfo;this.donationFlowModalManager.showConfirmationStepModal({donationType:n,amount:r,currencyType:`USD`,confirmDonationCB:()=>{this.payPalPaymentConfirmed(e,t)},cancelDonationCB:()=>{this.donationFlowModalManager.closeModal(),this.payPalPaymentCancelled(e,{})}})})}payPalPaymentConfirmed(e,t){return f(this,void 0,void 0,function*(){this.emitter.emit(`payPalPaymentConfirmed`,e,{}),this.donationFlowModalManager.showProcessingModal();let n=e.donationInfo.donationType,r=t?.details,i=new v({email:r?.email,firstName:r?.firstName,lastName:r?.lastName}),a=r.shippingAddress,o=new _({streetAddress:a?.line1,extendedAddress:a?.line2,locality:a?.city,region:a?.state,postalCode:a?.postalCode,countryCodeAlpha2:a?.countryCode}),s=this.upsellButtonDataSourceContainer?this.upsellButtonDataSourceContainer.oneTimeSuccessResponse.transaction_id:void 0,c=yield this.braintreeManager.submitDonation({nonce:t.nonce,paymentProvider:y.PayPal,donationInfo:e.donationInfo,customerInfo:i,billingInfo:o,upsellOnetimeTransactionId:s});if(!c.success){let e=c.value;this.donationFlowModalManager.showErrorModal({message:e.message});return}let l=c.value;switch(n){case x.OneTime:this.showUpsellModal(t,l);break;case x.Monthly:this.donationFlowModalManager.showThankYouModal({successResponse:l});break;case x.Upsell:this.upsellButtonDataSourceContainer?this.donationFlowModalManager.showThankYouModal({successResponse:this.upsellButtonDataSourceContainer.oneTimeSuccessResponse,upsellSuccessResponse:l}):this.donationFlowModalManager.showErrorModal({message:`Error setting up monthly donation`});break}})}payPalPaymentCancelled(e,t){return f(this,void 0,void 0,function*(){this.emitter.emit(`payPalPaymentCancelled`,e,t)})}payPalPaymentError(e,t){return f(this,void 0,void 0,function*(){this.emitter.emit(`payPalPaymentError`,e,t),console.error(`PaymentSector:payPalPaymentError error:`,e,e.donationInfo,t)})}renderPayPalButton(e){return f(this,void 0,void 0,function*(){let t=yield this.braintreeManager?.paymentProviders.paypalHandler.get();this.buttonDataSource=yield t?.renderPayPalButton({selector:`#paypal-button`,style:{color:`blue`,label:`paypal`,shape:`rect`,size:`medium`,tagline:!1},donationInfo:e}),this.buttonDataSource&&(this.buttonDataSource.delegate=this)})}showUpsellModal(e,t){return f(this,void 0,void 0,function*(){this.donationFlowModalManager.showUpsellModal({oneTimeAmount:t.amount,amountChanged:this.upsellAmountChanged.bind(this),noSelected:()=>{this.donationFlowModalManager.showThankYouModal({successResponse:t})},ctaMode:k.PayPalUpsellSlot,userClosedModalCallback:()=>{this.donationFlowModalManager.showThankYouModal({successResponse:t})}});let n=new b({amount:Te.getDefaultUpsellAmount(t.amount),donationType:x.Upsell,coverFees:!1});this.upsellButtonDataSourceContainer||this.renderUpsellPayPalButton({donationInfo:n,oneTimePayload:e,oneTimeSuccessResponse:t})})}upsellAmountChanged(e){this.upsellButtonDataSourceContainer&&(this.upsellButtonDataSourceContainer.upsellButtonDataSource.donationInfo.amount=e)}renderUpsellPayPalButton(e){return f(this,void 0,void 0,function*(){let t=yield(yield this.braintreeManager?.paymentProviders.paypalHandler.get())?.renderPayPalButton({selector:`#paypal-upsell-button`,style:{color:`blue`,label:`paypal`,shape:`rect`,size:`responsive`,tagline:!1},donationInfo:e.donationInfo});t?(t.delegate=this,this.upsellButtonDataSourceContainer=new Ee({upsellButtonDataSource:t,oneTimePayload:e.oneTimePayload,oneTimeSuccessResponse:e.oneTimeSuccessResponse})):console.error(`error rendering paypal upsell button`)})}},Oe=class{constructor(e){this.braintreeManager=e.braintreeManager,this.donationFlowModalManager=e.donationFlowModalManager}paymentInitiated(e,t){return f(this,void 0,void 0,function*(){this.donationFlowModalManager.showProcessingModal();let n=yield this.braintreeManager?.paymentProviders.applePayHandler.get();this.applePayDataSource=yield n?.createPaymentRequest(t,e),this.applePayDataSource&&(this.applePayDataSource.delegate=this)})}modalYesSelected(e,t){return f(this,void 0,void 0,function*(){this.donationFlowModalManager.showProcessingModal();let n=yield this.braintreeManager.submitUpsellDonation({oneTimeDonationResponse:e,amount:t});if(n.success)this.donationFlowModalManager.showThankYouModal({successResponse:e,upsellSuccessResponse:n.value});else{let e=n.value;this.donationFlowModalManager.showErrorModal({message:e.message})}})}paymentComplete(e){if(e.success){let t=e.value;this.applePayDataSource?.donationInfo.donationType==x.OneTime?this.donationFlowModalManager.showUpsellModal({oneTimeAmount:t.amount,yesSelected:this.modalYesSelected.bind(this,t),noSelected:this.donationFlowModalManager.showThankYouModal.bind(this.donationFlowModalManager,{successResponse:t}),userClosedModalCallback:this.donationFlowModalManager.showThankYouModal.bind(this.donationFlowModalManager,{successResponse:t})}):this.donationFlowModalManager.showThankYouModal({successResponse:t})}else{let t=e.value;this.donationFlowModalManager.showErrorModal({message:t.message})}}paymentFailed(){this.donationFlowModalManager.showErrorModal({message:`Payment failed`})}paymentCancelled(){this.donationFlowModalManager.closeModal()}},ke=class{constructor(e){this.contactInfo=e.contactInfo,this.donationInfo=e.donationInfo}},Ae=class{constructor(e){this.persistanceKey=`venmoRestorationStateInfo`,this.storageSystem=e?.storageSystem??this.getAvailableStorageSystem(()=>localStorage)??this.getAvailableStorageSystem(()=>sessionStorage)}clearState(){var e;(e=this.storageSystem)==null||e.removeItem(this.persistanceKey)}persistState(e,t){var n;let r=new ke({contactInfo:e,donationInfo:t}),i=JSON.stringify(r);(n=this.storageSystem)==null||n.setItem(this.persistanceKey,i)}getRestorationState(){return f(this,void 0,void 0,function*(){let e=this.storageSystem?.getItem(this.persistanceKey);if(!e){console.error(`restoreState: No stored data`);return}let t=JSON.parse(e);if(!t){console.error(`restoreState: Data could not be deserialized`);return}return new ke(t)})}getAvailableStorageSystem(e){try{let t=e();return t.setItem(`foo`,`bar`),t.removeItem(`foo`),t}catch{return}}},je=class{constructor(e){this.braintreeManager=e.braintreeManager,this.donationFlowModalManager=e.donationFlowModalManager,this.restorationStateHandler=e.restorationStateHandler??new Ae}startup(){return f(this,void 0,void 0,function*(){if((yield(yield this.braintreeManager.paymentProviders.venmoHandler.get())?.instance.get())?.hasTokenizationResult()){let e=yield this.restorationStateHandler.getRestorationState();e?this.paymentInitiated(e.contactInfo,e.donationInfo):(console.error(`no restoration info`),this.donationFlowModalManager.showErrorModal({message:`Error restoring donation session`}))}})}paymentInitiated(e,t){return f(this,void 0,void 0,function*(){this.restorationStateHandler.persistState(e,t);try{let n=yield(yield this.braintreeManager.paymentProviders.venmoHandler.get())?.startPayment();if(!n){this.restorationStateHandler.clearState(),this.donationFlowModalManager.showErrorModal({message:`Error setting up the donation`});return}this.handleTokenizationResult(n,e,t)}catch(e){this.restorationStateHandler.clearState(),this.handleTokenizationError(e),this.donationFlowModalManager.showErrorModal({message:`There was a problem loading your donation information. Please try again.`})}})}handleTokenizationResult(e,t,n){return f(this,void 0,void 0,function*(){this.restorationStateHandler.clearState(),this.donationFlowModalManager.startDonationSubmissionFlow({nonce:e.nonce,paymentProvider:y.Venmo,donationInfo:n,customerInfo:t.customer,billingInfo:t.billing})})}handleTokenizationError(e){switch(e.code){case`VENMO_APP_CANCELED`:break;case`VENMO_CANCELED`:break;default:console.error(`Error!`,e)}}},Me=class{constructor(e){this.emitter=h(),this.braintreeManager=e.braintreeManager,this.donationFlowModalManager=e.donationFlowModalManager}on(e,t){return this.emitter.on(e,t)}paymentInitiated(e){return f(this,void 0,void 0,function*(){let t=yield this.braintreeManager?.paymentProviders.googlePayHandler.get(),n=yield t.instance.get(),r=yield n.createPaymentDataRequest({emailRequired:!0,transactionInfo:{currencyCode:`USD`,totalPriceStatus:`FINAL`,totalPrice:`${e.total}`}}),i=r.allowedPaymentMethods[0];i.parameters.billingAddressRequired=!0,i.parameters.billingAddressParameters={format:`FULL`,phoneNumberRequired:!1};try{let i=yield t.paymentsClient.loadPaymentData(r),a=yield n.parseResponse(i),o=i.paymentMethodData.info?.billingAddress,s=o?.name,c=s,l=``,u=s?.lastIndexOf(` `);u&&u!==-1&&(c=s?.substr(0,u),l=s?.substr(u));let d=new v({email:i.email,firstName:c,lastName:l}),f=new _({streetAddress:o?.address1,extendedAddress:o?.address2,locality:o?.locality,region:o?.administrativeArea,postalCode:o?.postalCode,countryCodeAlpha2:o?.countryCode});this.donationFlowModalManager.startDonationSubmissionFlow({nonce:a.nonce,paymentProvider:y.GooglePay,bin:a.details.bin,binName:a.binData.issuingBank,donationInfo:e,customerInfo:d,billingInfo:f})}catch{this.emitter.emit(`paymentCancelled`),this.donationFlowModalManager.closeModal()}})}},Ne=class{startup(){return f(this,void 0,void 0,function*(){var e,t;(e=this.venmoHandler)==null||e.startup(),(t=this.creditCardHandler)==null||t.startup()})}showUpsellModal(e){return f(this,void 0,void 0,function*(){return this.donationFlowModalManager.showUpsellModal(e)})}showConfirmationStepModal(e){return this.donationFlowModalManager.showConfirmationStepModal(e)}get creditCardHandler(){return this.creditCardHandlerCache||(this.creditCardHandlerCache=new Se({braintreeManager:this.braintreeManager,donationFlowModalManager:this.donationFlowModalManager,recaptchaManager:this.recaptchaManager})),this.creditCardHandlerCache}get paypalHandler(){return this.paypalHandlerCache||(this.paypalHandlerCache=new De({braintreeManager:this.braintreeManager,donationFlowModalManager:this.donationFlowModalManager})),this.paypalHandlerCache}get applePayHandler(){return this.applePayHandlerCache||(this.applePayHandlerCache=new Oe({braintreeManager:this.braintreeManager,donationFlowModalManager:this.donationFlowModalManager})),this.applePayHandlerCache}get venmoHandler(){return this.venmoHandlerCache||(this.venmoHandlerCache=new je({braintreeManager:this.braintreeManager,donationFlowModalManager:this.donationFlowModalManager})),this.venmoHandlerCache}get googlePayHandler(){return this.googlePayHandlerCache||(this.googlePayHandlerCache=new Me({braintreeManager:this.braintreeManager,donationFlowModalManager:this.donationFlowModalManager})),this.googlePayHandlerCache}constructor(e){this.braintreeManager=e.braintreeManager,this.modalManager=e.modalManager,this.recaptchaManager=e.recaptchaManager,this.resources=e.resources,this.donationFlowModalManager=new Te({braintreeManager:this.braintreeManager,modalManager:this.modalManager,analytics:this.resources.analytics})}},Pe=class{constructor(e){this.isExecuting=!1,this.grecaptchaLibrary=e.grecaptchaLibrary,this.siteKey=e.siteKey}execute(){return this.isExecuting&&this.finishExecution(),this.isExecuting=!0,new Promise((e,t)=>{this.executionSuccessBlock=t=>{this.finishExecution(),e(t)},this.executionExpiredBlock=()=>{this.finishExecution(),t(`expired`)},this.executionErrorBlock=()=>{this.finishExecution(),t(`error`)},this.grecaptchaLibrary.execute()})}finishExecution(){this.isExecuting=!1,this.grecaptchaLibrary.reset()}setup(e,t,n,r){this.grecaptchaLibrary.render(e,{callback:this.responseHandler.bind(this),"expired-callback":this.expiredHandler.bind(this),"error-callback":this.errorHandler.bind(this),sitekey:this.siteKey,tabindex:t,theme:n,type:r,size:`invisible`})}responseHandler(e){this.executionSuccessBlock&&(this.executionSuccessBlock(e),this.executionSuccessBlock=void 0)}expiredHandler(){this.executionExpiredBlock&&(this.executionExpiredBlock(),this.executionExpiredBlock=void 0)}errorHandler(){this.executionErrorBlock&&(this.executionErrorBlock(),this.executionErrorBlock=void 0)}},Fe=class{constructor(e){this.hostedFieldFieldOptions=e.hostedFieldFieldOptions,this.hostedFieldStyle=e.hostedFieldStyle,this.hostedFieldContainer=e.hostedFieldContainer}},N;(function(e){e.LeaveSpace=`leave-space`,e.CompressSpace=`compress-space`})(N||(N={}));var P=class extends c{constructor(){super(...arguments),this.error=!1,this.required=!1,this.iconSpaceOption=N.LeaveSpace,this.requiredIndicatorSpaceOption=N.LeaveSpace}render(){return a`
      <div class="input-wrapper ${this.errorClass} ${this.iconSpaceOptionClass}">
        <div class="icon-container">${this.icon}</div>
        <div class="required-indicator ${this.requiredIndicatorSpaceOption}">
          ${this.required?a` * `:i}
        </div>

        <slot></slot>
      </div>
    `}get errorClass(){return this.error?`error`:``}get iconSpaceOptionClass(){return this.iconSpaceOption===N.CompressSpace?`compress-space`:``}static get styles(){let e=n`var(--inputBorder, 1px solid #d9d9d9)`,t=n`var(--badgedInputBorderErrorColor, red)`,r=n`var(--badgedInputIconSize, 20px)`,i=n`var(--badgedInputIconColor, #2c2c2c)`,a=n`var(--badgedInputIconSpacerWidth, 3rem)`,o=n`var(--badgedInputNoIconSpacerWidth, 1rem)`,s=n`var(--badgedInputHeight, 3rem)`,c=n`var(--badgedInputRequiredIndicatorColor, red)`,l=n`var(--badgedInputRequiredIndicatorMargin, 0 0.25rem 0 0)`;return n`
      .input-wrapper {
        border: ${e};
        height: ${s};
        display: flex;
        align-items: center;
      }

      .input-wrapper.error {
        box-shadow: inset 0px 0px 0px 1px ${t};
        border-color: ${t};
      }

      .input-wrapper.compress-space .icon-container {
        width: ${o};
      }

      .icon-container {
        width: ${a};
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
      }

      .icon-container svg {
        height: ${r};
        width: ${r};
        fill: ${i};
      }

      .required-indicator {
        color: ${c};
        font-size: ${n`var(--badgedInputRequiredIndicatorFontSize, 2rem)`};
        margin: ${l};
      }

      .required-indicator.leave-space {
        width: 0.5em;
      }
    `}};p([r({type:Boolean})],P.prototype,`error`,void 0),p([r({type:Object})],P.prototype,`icon`,void 0),p([r({type:Boolean})],P.prototype,`required`,void 0),p([r({type:String})],P.prototype,`iconSpaceOption`,void 0),p([r({type:String})],P.prototype,`requiredIndicatorSpaceOption`,void 0),P=p([u(`badged-input`)],P);var F={AF:`Afghanistan`,AX:`Aland Islands`,AL:`Albania`,DZ:`Algeria`,AS:`American Samoa`,AD:`Andorra`,AO:`Angola`,AI:`Anguilla`,AQ:`Antarctica`,AG:`Antigua and Barbuda`,AR:`Argentina`,AM:`Armenia`,AW:`Aruba`,AU:`Australia`,AT:`Austria`,AZ:`Azerbaijan`,BS:`Bahamas`,BH:`Bahrain`,BD:`Bangladesh`,BB:`Barbados`,BY:`Belarus`,BE:`Belgium`,BZ:`Belize`,BJ:`Benin`,BM:`Bermuda`,BT:`Bhutan`,BO:`Bolivia`,BQ:`Bonaire, Saint Eustatius and Saba `,BA:`Bosnia and Herzegovina`,BW:`Botswana`,BV:`Bouvet Island`,BR:`Brazil`,IO:`British Indian Ocean Territory`,VG:`British Virgin Islands`,BN:`Brunei`,BG:`Bulgaria`,BF:`Burkina Faso`,BI:`Burundi`,KH:`Cambodia`,CM:`Cameroon`,CA:`Canada`,CV:`Cape Verde`,KY:`Cayman Islands`,CF:`Central African Republic`,TD:`Chad`,CL:`Chile`,CN:`China`,CX:`Christmas Island`,CC:`Cocos Islands`,CO:`Colombia`,KM:`Comoros`,CK:`Cook Islands`,CR:`Costa Rica`,HR:`Croatia`,CU:`Cuba`,CW:`Curacao`,CY:`Cyprus`,CZ:`Czech Republic`,CD:`Democratic Republic of the Congo`,DK:`Denmark`,DJ:`Djibouti`,DM:`Dominica`,DO:`Dominican Republic`,TL:`East Timor`,EC:`Ecuador`,EG:`Egypt`,SV:`El Salvador`,GQ:`Equatorial Guinea`,ER:`Eritrea`,EE:`Estonia`,ET:`Ethiopia`,FK:`Falkland Islands`,FO:`Faroe Islands`,FJ:`Fiji`,FI:`Finland`,FR:`France`,GF:`French Guiana`,PF:`French Polynesia`,TF:`French Southern Territories`,GA:`Gabon`,GM:`Gambia`,GE:`Georgia`,DE:`Germany`,GH:`Ghana`,GI:`Gibraltar`,GR:`Greece`,GL:`Greenland`,GD:`Grenada`,GP:`Guadeloupe`,GU:`Guam`,GT:`Guatemala`,GG:`Guernsey`,GN:`Guinea`,GW:`Guinea-Bissau`,GY:`Guyana`,HT:`Haiti`,HM:`Heard Island and McDonald Islands`,HN:`Honduras`,HK:`Hong Kong`,HU:`Hungary`,IS:`Iceland`,IN:`India`,ID:`Indonesia`,IR:`Iran`,IQ:`Iraq`,IE:`Ireland`,IM:`Isle of Man`,IL:`Israel`,IT:`Italy`,CI:`Ivory Coast`,JM:`Jamaica`,JP:`Japan`,JE:`Jersey`,JO:`Jordan`,KZ:`Kazakhstan`,KE:`Kenya`,KI:`Kiribati`,XK:`Kosovo`,KW:`Kuwait`,KG:`Kyrgyzstan`,LA:`Laos`,LV:`Latvia`,LB:`Lebanon`,LS:`Lesotho`,LR:`Liberia`,LY:`Libya`,LI:`Liechtenstein`,LT:`Lithuania`,LU:`Luxembourg`,MO:`Macao`,MK:`Macedonia`,MG:`Madagascar`,MW:`Malawi`,MY:`Malaysia`,MV:`Maldives`,ML:`Mali`,MT:`Malta`,MH:`Marshall Islands`,MQ:`Martinique`,MR:`Mauritania`,MU:`Mauritius`,YT:`Mayotte`,MX:`Mexico`,FM:`Micronesia`,MD:`Moldova`,MC:`Monaco`,MN:`Mongolia`,ME:`Montenegro`,MS:`Montserrat`,MA:`Morocco`,MZ:`Mozambique`,MM:`Myanmar`,NA:`Namibia`,NR:`Nauru`,NP:`Nepal`,NL:`Netherlands`,NC:`New Caledonia`,NZ:`New Zealand`,NI:`Nicaragua`,NE:`Niger`,NG:`Nigeria`,NU:`Niue`,NF:`Norfolk Island`,KP:`North Korea`,MP:`Northern Mariana Islands`,NO:`Norway`,OM:`Oman`,PK:`Pakistan`,PW:`Palau`,PS:`Palestinian Territory`,PA:`Panama`,PG:`Papua New Guinea`,PY:`Paraguay`,PE:`Peru`,PH:`Philippines`,PN:`Pitcairn`,PL:`Poland`,PT:`Portugal`,PR:`Puerto Rico`,QA:`Qatar`,CG:`Republic of the Congo`,RE:`Reunion`,RO:`Romania`,RU:`Russia`,RW:`Rwanda`,BL:`Saint Barthelemy`,SH:`Saint Helena`,KN:`Saint Kitts and Nevis`,LC:`Saint Lucia`,MF:`Saint Martin`,PM:`Saint Pierre and Miquelon`,VC:`Saint Vincent and the Grenadines`,WS:`Samoa`,SM:`San Marino`,ST:`Sao Tome and Principe`,SA:`Saudi Arabia`,SN:`Senegal`,RS:`Serbia`,SC:`Seychelles`,SL:`Sierra Leone`,SG:`Singapore`,SX:`Sint Maarten`,SK:`Slovakia`,SI:`Slovenia`,SB:`Solomon Islands`,SO:`Somalia`,ZA:`South Africa`,GS:`South Georgia and the South Sandwich Islands`,KR:`South Korea`,SS:`South Sudan`,ES:`Spain`,LK:`Sri Lanka`,SD:`Sudan`,SR:`Suriname`,SJ:`Svalbard and Jan Mayen`,SZ:`Swaziland`,SE:`Sweden`,CH:`Switzerland`,SY:`Syria`,TW:`Taiwan`,TJ:`Tajikistan`,TZ:`Tanzania`,TH:`Thailand`,TG:`Togo`,TK:`Tokelau`,TO:`Tonga`,TT:`Trinidad and Tobago`,TN:`Tunisia`,TR:`Turkey`,TM:`Turkmenistan`,TC:`Turks and Caicos Islands`,TV:`Tuvalu`,VI:`U.S. Virgin Islands`,UG:`Uganda`,UA:`Ukraine`,AE:`United Arab Emirates`,GB:`United Kingdom`,UM:`United States Minor Outlying Islands`,US:`United States`,UY:`Uruguay`,UZ:`Uzbekistan`,VU:`Vanuatu`,VA:`Vatican`,VE:`Venezuela`,VN:`Vietnam`,WF:`Wallis and Futuna`,EH:`Western Sahara`,YE:`Yemen`,ZM:`Zambia`,ZW:`Zimbabwe`},I=class extends c{constructor(){super(...arguments),this.selectedCountry=`US`,this.donorEmail=``,this.minTwoCharPattern=`.*\\S{2,}.*`,this.minTwoCharValidationMessage=o(`Enter at least two characters`),this.streetAddressPattern=`.*?\\S.{2,}\\S.*?`,this.streetAddressValidationMessage=o(`Enter at least four characters`),this.usZipCodePattern=`^\\d{5}(-?\\d{4})?$`,this.usZipCodeValidationMessage=o(`Enter a valid 5 or 9 digit zip/postal code`)}updated(e){e.has(`donorEmail`)&&(this.emailField.value=this.donorEmail??``)}reportValidity(){return this.validateFormFields(),this.validateForm()}validateFormFields(){[{badgedInput:this.emailBadgedInput,inputField:this.emailField},{badgedInput:this.firstNameBadgedInput,inputField:this.firstNameField},{badgedInput:this.lastNameBadgedInput,inputField:this.lastNameField},{badgedInput:this.streetAddressBadgedInput,inputField:this.streetAddressField},{badgedInput:this.localityBadgedInput,inputField:this.localityField},{badgedInput:this.regionBadgedInput,inputField:this.regionField},{badgedInput:this.postalBadgedInput,inputField:this.postalCodeField}].forEach(({badgedInput:e,inputField:t})=>{e.error=!t.checkValidity()})}validateForm(){let e=this.form.reportValidity();return e?this.errorMessage.innerText=``:this.errorMessage.innerText=o(`Please enter any missing or invalid contact information below`),e}focus(){this.emailField.focus()}render(){return a`
      <div id="donation-contact-form-error-message"></div>
      <form>
        <div class="row">
          ${this.generateInput({id:`donation-contact-form-email`,label:`Email`,required:!0,fieldType:`email`,name:`email`,autocomplete:`email`,minlength:5,maxlength:255})}
        </div>

        <div class="row">
          ${this.generateInput({id:`donation-contact-form-first-name`,label:`First name`,name:`fname`,required:!0,validationPattern:this.minTwoCharPattern,validationMessage:this.minTwoCharValidationMessage,maxlength:255,autocomplete:`given-name`})}
          ${this.generateInput({id:`donation-contact-form-last-name`,label:`Last name`,name:`lname`,autocomplete:`family-name`,required:!0,validationPattern:this.minTwoCharPattern,validationMessage:this.minTwoCharValidationMessage,maxlength:255})}
        </div>

        <div class="row">
          ${this.generateInput({id:`donation-contact-form-street-address`,label:`Address`,required:!0,autocomplete:`address-line1`,name:`street-address`,validationPattern:this.streetAddressPattern,validationMessage:this.streetAddressValidationMessage})}
        </div>
        <div class="row">
          ${this.generateInput({id:`donation-contact-form-locality`,label:`City`,autocomplete:`address-level2`,required:!0,name:`locality`,validationPattern:this.minTwoCharPattern,validationMessage:this.minTwoCharValidationMessage})}
        </div>
        <div class="row">${this.countrySelectorTemplate}</div>
        <div class="row region-postal-row">
          ${this.generateInput({id:`donation-contact-form-region`,label:`State / Province`,autocomplete:`address-level1`,required:this.regionAndPostalCodeRequired,name:`region`,validationPattern:this.regionAndPostalCodeRequired?this.minTwoCharPattern:void 0,validationMessage:this.regionAndPostalCodeRequired?this.minTwoCharValidationMessage:void 0})}
          ${this.generateInput({id:`donation-contact-form-postal-code`,label:`Zip / Postal Code`,autocomplete:`postal-code`,required:this.regionAndPostalCodeRequired,name:`postal`,validationPattern:this.regionAndPostalCodeRequired?this.usZipCodePattern:void 0,validationMessage:this.regionAndPostalCodeRequired?this.usZipCodeValidationMessage:void 0})}
        </div>
      </form>
      ${this.getStyles}
    `}get regionAndPostalCodeRequired(){return this.selectedCountry===`US`}get countrySelectorTemplate(){return a`
      <div class="field">
        <label for="donation-contact-form-countryCodeAlpha2" class="field-label">
          Country<span class="required-asterisk"> *</span>
        </label>
        <badged-input .iconSpaceOption=${N.CompressSpace}>
          <select
            id="donation-contact-form-countryCodeAlpha2"
            @change=${e=>{let t=e.target.value;F[t]&&(this.selectedCountry=t)}}
          >
            ${Object.keys(F).map(e=>{let t=F[e];return a`
                <option value=${e} ?selected=${e===this.selectedCountry}>${t}</option>
              `})}
          </select>
        </badged-input>
      </div>
    `}createRenderRoot(){return this}inputFocused(e){this.errorMessage.innerText=``;let t=e.target.id,n=this.querySelector(`badged-input.${t}`);n.error=!1}generateInput(e){let t=e.required??!0,n=e.fieldType??`text`;return a`
      <div class="field ${e.id}">
        <label for=${e.id} class="field-label">
          ${e.label}${t?a`<span class="required-asterisk"> *</span>`:i}
        </label>
        <badged-input
          class=${e.id}
          .iconSpaceOption=${N.CompressSpace}
          .requiredIndicatorSpaceOption=${N.CompressSpace}
        >
          <input
            type=${n}
            id=${e.id}
            class="donation-contact-form-input"
            name=${e.name}
            maxlength=${l(e.maxlength)}
            minlength=${l(e.minlength)}
            autocomplete=${e.autocomplete??`on`}
            pattern=${l(e.validationPattern)}
            title=${l(e.validationMessage)}
            @focus=${this.inputFocused}
            ?required=${t}
          />
        </badged-input>
      </div>
    `}get donorContactInfo(){return new te({billing:this.billingInfo,customer:this.contactInfo})}get billingInfo(){return new _({streetAddress:this.streetAddressField.value,locality:this.localityField.value,region:this.regionField.value,postalCode:this.postalCodeField.value,countryCodeAlpha2:this.countryCodeAlpha2Field.value})}get contactInfo(){return new v({email:this.emailField.value,firstName:this.firstNameField.value,lastName:this.lastNameField.value})}get getStyles(){let e=n`var(--badgedInputNoIconSpacerWidth, 3rem)`,t=n`var(--fieldRowGap, 5px)`,r=n`var(--fontFamily, "Helvetica Neue", Helvetica, Arial, sans-serif)`,i=n`var(--contactFieldFontSize, 14px)`,o=n`var(--inputFieldFontColor, #2c2c2c)`,s=n`var(--fieldLabelFontFamily, "Helvetica Neue", Helvetica, Arial, sans-serif)`,c=n`var(--fieldLabelFontSize, 14px)`,l=n`var(--fieldLabelColor, #2c2c2c)`,u=n`var(--fieldLabelMarginBottom, 5px)`,d=n`var(--badgedInputRequiredIndicatorColor, red)`,f=n`calc(100% - ${e})`;return a`
      <style>
        /*
          **NOTE**
          This element is in the lightDOM so be sure to prefix all styles
          with "contact-form" so styles don't leak.
         */
        /*
          Grid (not flex) so that a label wrapping to two lines in one column
          doesn't push that column's input out of alignment with its siblings -
          all labels share row-line 1 and all inputs share row-line 2, each
          sized to the tallest content in that line.
        */
        contact-form .row {
          display: grid;
          grid-auto-flow: column;
          grid-auto-columns: 1fr;
          grid-template-rows: auto auto;
          column-gap: ${t};
        }

        /*
          Per WEBDEV-8310 QA feedback: State/Province gives up space to
          Zip/Postal Code so "Zip / Postal Code" stops wrapping to two lines.
          30px (not 10px) is what "Zip / Postal Code *" actually needs at this
          width, measured against its rendered label width plus a safety margin.
        */
        contact-form .row.region-postal-row {
          grid-template-columns: calc(60% - 30px) calc(40% + 30px);
        }

        /* uniform vertical rhythm between every field, regardless of grouping */
        contact-form .row + .row {
          margin-top: ${t};
        }

        contact-form .field {
          display: contents;
        }

        contact-form .field-label {
          display: block;
          font-family: ${s};
          font-size: ${c};
          font-weight: bold;
          color: ${l};
          margin-bottom: ${u};
        }

        contact-form .required-asterisk {
          color: ${d};
        }

        contact-form badged-input.donation-contact-form-region,
        contact-form badged-input.donation-contact-form-postal-code {
          width: 100%;
        }

        contact-form #donation-contact-form-error-message {
          color: red;
          font-size: 1.4rem;
          margin-bottom: 0.6rem;
        }

        contact-form .donation-contact-form-input {
          width: ${f};
          border: 0;
          outline: 0;
          background: transparent;
          font-weight: bold;
          color: ${o};
          font-size: ${i};
          padding: 0;
          font-family: ${r};
        }

        contact-form #donation-contact-form-countryCodeAlpha2 {
          width: ${f};
          height: 100%;
          box-sizing: border-box;
          font-weight: bold;
          font-size: ${i};
          color: ${o};
          font-family: ${r};
          border: 0;
          background: #fff;
        }
      </style>
    `}};p([d(`badged-input.donation-contact-form-email`)],I.prototype,`emailBadgedInput`,void 0),p([d(`#donation-contact-form-email`)],I.prototype,`emailField`,void 0),p([d(`badged-input.donation-contact-form-first-name`)],I.prototype,`firstNameBadgedInput`,void 0),p([d(`#donation-contact-form-first-name`)],I.prototype,`firstNameField`,void 0),p([d(`badged-input.donation-contact-form-last-name`)],I.prototype,`lastNameBadgedInput`,void 0),p([d(`#donation-contact-form-last-name`)],I.prototype,`lastNameField`,void 0),p([d(`badged-input.donation-contact-form-postal-code`)],I.prototype,`postalBadgedInput`,void 0),p([d(`#donation-contact-form-postal-code`)],I.prototype,`postalCodeField`,void 0),p([d(`badged-input.donation-contact-form-street-address`)],I.prototype,`streetAddressBadgedInput`,void 0),p([d(`#donation-contact-form-street-address`)],I.prototype,`streetAddressField`,void 0),p([d(`badged-input.donation-contact-form-locality`)],I.prototype,`localityBadgedInput`,void 0),p([d(`#donation-contact-form-locality`)],I.prototype,`localityField`,void 0),p([d(`badged-input.donation-contact-form-region`)],I.prototype,`regionBadgedInput`,void 0),p([d(`#donation-contact-form-region`)],I.prototype,`regionField`,void 0),p([d(`#donation-contact-form-countryCodeAlpha2`)],I.prototype,`countryCodeAlpha2Field`,void 0),p([d(`#donation-contact-form-error-message`)],I.prototype,`errorMessage`,void 0),p([d(`form`)],I.prototype,`form`,void 0),p([r({type:String})],I.prototype,`selectedCountry`,void 0),p([r({type:String})],I.prototype,`donorEmail`,void 0),I=p([u(`contact-form`)],I);var Ie=a`
<svg viewBox="0 0 12 12" xmlns="http://www.w3.org/2000/svg" aria-labelledby="creditCardTitleID creditCardDescID">
  <title id="creditCardTitleID">Credit card icon</title>
  <desc id="creditCardDescID">An illustration of a credit card</desc>
  <g class="fill-color" fill-rule="evenodd" transform="translate(0 2)">
    <g fill-rule="nonzero">
      <path d="m11.998 0v9h-11.998v-9z" />
      <g fill="#fff">
        <path d="m11.143 3.429h-10.286v4.714h10.286z" />
        <path d="m11.143.857h-10.286v1.286h10.286z" />
      </g>
    </g>
    <g>
      <path d="m8.143 6.429h1v1h-1z" />
      <path d="m9.429 6.429h1v1h-1z" />
    </g>
  </g>
</svg>
`,Le=a`
<svg viewBox="0 0 12 12" xmlns="http://www.w3.org/2000/svg" aria-labelledby="calendarTitleID calendarDescID">
  <title id="calendarTitleID">Calendar icon</title>
  <desc id="calendarDescID">An illustration of a calendar</desc>

  <g class="fill-color" fill-rule="evenodd">
    <path d="m11.998.857v11h-11.998v-11z" fill-rule="nonzero" />
    <path d="m11.143 3h-10.286v8h10.286z" fill="#fff" fill-rule="nonzero" />
    <path d="m9 0h1v1h-1z" />
    <path d="m2.143 0h1v1h-1z" />
    <path d="m2.143.857h1v1h-1z" fill="#fff" />
    <path d="m9 .857h1v1h-1z" fill="#fff" />
    <path
      d="m4.92342857 9.14285714v-4.2h-.678c-.02400012.1600008-.07399962.29399946-.15.402s-.16899945.19499967-.279.261-.23399931.11199987-.372.138-.28099926.03700002-.429.033v.642h1.056v2.724zm3.336 0h-.852v-2.724h-1.056v-.642c.14800074.00400002.29099931-.00699987.429-.033s.26199945-.07199967.372-.138.20299962-.15299946.279-.261.12599988-.2419992.15-.402h.678z"
      fill-rule="nonzero"
    />
  </g>
</svg>
`,L=a`
<svg viewBox="0 0 12 12" xmlns="http://www.w3.org/2000/svg" aria-labelledby="lockTitleID lockDescID">
  <title id="lockTitleID">Lock icon</title>
  <desc id="lockDescID">An illustration of a lock</desc>
  <path
    d="m9.8480234 5.66075891v-2.17912633c-.00688261-.97492716-.37725298-1.79574705-1.11111111-2.46245966s-1.63734389-1.00632179-2.71045726-1.01882754c-1.04529617-.01250574-1.94175593.31459769-2.68937928.9813103-.74762335.66671262-1.13190232 1.4842758-1.15283692 2.45268954v2.22641369c-.04846504.00625288-.10037138.01250575-.15571902.01875862-.05534764.00625288-.09348877.00937931-.11442337.00937931-.35302046.00625288-.59362498.06917241-.72181356.18875862-.12818859.1195862-.19228288.33022987-.19228288.631931v4.73576994c0 .5030957.269999.7546436.80999699.7546436h8.36968211c.2839076 0 .491533-.0597931.6228761-.1793793s.197158-.3082145.1974448-.565885v-4.82057452c0-.25793103-.0640943-.44499615-.1922829-.56119538s-.3340933-.17755555-.6177141-.18406896c-.0415824 0-.102092-.00468965-.1815288-.01406896-.07943676-.00937931-.13306375-.01406897-.16088096-.01406897zm-1.85873446.00937931h-3.92523766c-.01376522-.12583907-.02064783-.21077393-.02064783-.25480458l-.01032391-.97154019c0-.65420686.0034413-.9813103.01032391-.9813103.00688261-.49684289.1919961-.91513405.55534047-1.2548735.36334438-.33973945.81845687-.51273561 1.36533747-.51898848.52623277-.01875862.98492995.13691187 1.37609154.46701147.39116158.3300996.60050759.74044441.62803802 1.23103443.01376522.2076475.02064783.83032946.02064783 1.86804589v.41503446z"
    class="fill-color"
  />
</svg>
`,R=class extends c{get hostedFieldContainer(){return new xe({number:this.numberField,cvv:this.cvvField,expirationDate:this.expirationField,errorContainer:this.errorMessageField})}render(){return a`
      <div id="braintree-error-message"></div>
      <div class="braintree-row">
        <div class="field">
          <label class="field-label">Card Number<span class="required-asterisk"> *</span></label>
          <badged-input
            .icon=${Ie}
            .requiredIndicatorSpaceOption=${N.CompressSpace}
            class="creditcard"
          >
            <div class="braintree-input" id="braintree-creditcard"></div>
          </badged-input>
        </div>
      </div>
      <div class="braintree-row expiration-cvv-row">
        <div class="field">
          <label class="field-label"
            >Expiration (MM / YY)<span class="required-asterisk"> *</span></label
          >
          <badged-input
            .icon=${Le}
            .requiredIndicatorSpaceOption=${N.CompressSpace}
            class="expiration"
          >
            <div class="braintree-input" id="braintree-expiration"></div>
          </badged-input>
        </div>
        <div class="field">
          <label class="field-label">CVC<span class="required-asterisk"> *</span></label>
          <badged-input
            .icon=${L}
            .requiredIndicatorSpaceOption=${N.CompressSpace}
            class="cvv"
          >
            <div class="braintree-input" id="braintree-cvv"></div>
          </badged-input>
        </div>
      </div>
      ${this.getStyles}
    `}createRenderRoot(){return this}get getStyles(){let e=n`var(--fieldLabelFontFamily, "Helvetica Neue", Helvetica, Arial, sans-serif)`,t=n`var(--fieldLabelFontSize, 14px)`,r=n`var(--fieldLabelColor, #2c2c2c)`,i=n`var(--fieldLabelMarginBottom, 5px)`,o=n`var(--fieldRowGap, 5px)`;return a`
      <style>
        /*
          **NOTE**
          This element is in the lightDOM so be sure to prefix all styles
          with "credit-card-fields" so styles don't leak.
         */
        credit-card-fields .field-label {
          display: block;
          font-family: ${e};
          font-size: ${t};
          font-weight: bold;
          color: ${r};
          margin-bottom: ${i};
        }

        credit-card-fields .required-asterisk {
          color: ${n`var(--badgedInputRequiredIndicatorColor, red)`};
        }

        /*
          Grid (not flex) so that a label wrapping to two lines in one column
          doesn't push that column's input out of alignment with its sibling -
          both labels share row-line 1 and both inputs share row-line 2.
        */
        credit-card-fields .braintree-row {
          display: grid;
          grid-auto-flow: column;
          grid-auto-columns: 1fr;
          grid-template-rows: auto auto;
          column-gap: ${o};
        }

        /*
          Per WEBDEV-8310 QA feedback: CVC gives up space to Expiration so
          "Expiration (MM / YY)" stops wrapping to two lines. 25px (not 10px)
          is what it actually needs, measured against its rendered label
          width plus a safety margin - CVC's own label has plenty of room to
          spare.
        */
        credit-card-fields .braintree-row.expiration-cvv-row {
          grid-template-columns: calc(50% + 25px) calc(50% - 25px);
        }

        /* uniform vertical rhythm between every field, regardless of grouping */
        credit-card-fields .braintree-row + .braintree-row {
          margin-top: ${o};
        }

        credit-card-fields .field {
          display: contents;
        }

        credit-card-fields badged-input {
          width: 100%;
        }

        credit-card-fields .braintree-input {
          width: 100%;
          height: 100%;
        }

        credit-card-fields #braintree-error-message {
          color: red;
          font-size: 1.4rem;
          margin-bottom: 0.6rem;
        }
      </style>
    `}};p([d(`#braintree-creditcard`)],R.prototype,`numberField`,void 0),p([d(`#braintree-expiration`)],R.prototype,`expirationField`,void 0),p([d(`#braintree-cvv`)],R.prototype,`cvvField`,void 0),p([d(`#braintree-error-message`)],R.prototype,`errorMessageField`,void 0),R=p([u(`credit-card-fields`)],R);function Re(e){let t=e.trim();if(!/^[0-9]+([.,][0-9]*)*$/.test(t))return NaN;let n=Math.max(t.lastIndexOf(`,`),t.lastIndexOf(`.`));if(n===-1)return parseFloat(t);let r=t[n],i=r===`,`?`.`:`,`,a=t.length-n-1,o=(e,t)=>RegExp(`^[0-9]{1,3}(\\${t}[0-9]{3})*$`).test(e);if(a<=2){let e=t.slice(0,n);if(!(e.includes(i)?o(e,i):/^[0-9]+$/.test(e)))return NaN;let r=t.slice(n+1);return parseFloat(`${e.split(i).join(``)}.${r||`0`}`)}return a===3&&r===`,`&&o(t,r)?parseFloat(t.split(r).join(``)):NaN}var z;(function(e){e.HideBadge=`hidebadge`,e.ShowBadge=`showbadge`,e.HideBadgeLeaveSpacing=`hidebadgeleavespacing`})(z||(z={}));var B=class extends c{constructor(){super(...arguments),this.sectionBadge=`0`,this.badgeMode=z.ShowBadge}render(){return a`
      <div class="container ${this.badgeMode}">
        <div class="badge-container">
          <div class="badge">${this.sectionBadge}</div>
        </div>
        <div class="content-container">
          ${this.headline?a` <div class="title">${this.headline}</div> `:``}
          <div class="content">
            <slot></slot>
          </div>
        </div>
      </div>
    `}static get styles(){let e=n`var(--formSectionBadgeTransition, 0.25s ease-out)`,t=n`var(--formSectionBadgeMargin, 1rem)`,r=n`var(--formSectionBadgeBackgroundColor, #333)`,i=n`var(--formSectionBadgeRadius, 1.2rem)`,a=n`calc(${i} * 2)`,o=n`var(--formSectionBadgeFontSize, 1.8rem)`,s=n`var(--formSectionBadgeFontWeight, bold)`,c=n`var(--formSectionBadgeFontColor, #fff)`,l=n`var(--formSectionTitleFontSize, 1.8rem)`,u=n`var(--formSectionTitleFontWeight, bold)`;return n`
      :host {
        display: block;
        background-color: ${n`var(--formSectionContentBackgroundColor, transparent)`};
        color: ${n`var(--formSectionTextColor, #333)`};
      }
      .container {
        position: relative;
        padding: 0.5rem;
      }

      .content-container {
        position: relative;
        left: calc(${a} + ${t});
        width: calc(100% - (${a} + ${t}));
        transition: ${e};
        z-index: 1;
      }

      .hidebadge .content-container {
        left: 0;
        width: 100%;
      }

      .hidebadge .badge-container {
        display: none;
      }

      .hidebadgeleavespacing .badge {
        display: none;
      }

      .badge-container {
        position: absolute;
        width: ${a};
      }

      .badge {
        background-color: ${r};
        color: ${c};
        width: ${a};
        height: ${a};
        border-radius: ${i};
        display: flex;
        justify-content: center;
        align-items: center;
        font-weight: ${s};
        font-size: ${o};
      }

      .title {
        line-height: ${n`calc(${i} * 2)`};
        margin-bottom: 0.5rem;
        font-size: ${l};
        font-weight: ${u};
      }
    `}};p([r({type:String})],B.prototype,`sectionBadge`,void 0),p([r({type:String})],B.prototype,`headline`,void 0),p([r({type:String})],B.prototype,`badgeMode`,void 0),B=p([u(`donation-form-section`)],B);var V;(function(e){e.HideNumbers=`hidenumbers`,e.ShowNumbers=`shownumbers`})(V||(V={}));var H;(function(e){e.DonationType=`donationType`,e.Amount=`amount`})(H||(H={}));var U;(function(e){e.ValidDonationAmount=`valid_donation_amount`,e.InvalidDonationAmount=`invalid_donation_amount`,e.DonationTooHigh=`donation_too_high`,e.DonationTooLow=`donation_too_low`})(U||(U={}));var W;(function(e){e.Button=`button`,e.Checkbox=`checkbox`,e.Hide=`hide`})(W||(W={}));var G;(function(e){e.SingleLine=`single-line`,e.MultiLine=`multi-line`})(G||(G={}));var K=class extends c{constructor(){super(...arguments),this.donationInfo=C,this.stepNumberMode=V.ShowNumbers,this.amountOptions=S,this.amountSelectionLayout=G.MultiLine,this.frequencySelectionMode=W.Button,this.customAmountMode=`display`,this.coverFeesCheckboxMode=`display`,this.amountTitleDisplayMode=`default`,this.customAmountSelected=!1,this.currencyValidator=new Ce}render(){let e=this.amountTitleDisplayMode==="default"?`Choose an amount (USD)`:``;return a`
      ${this.frequencySelectionMode===W.Button?this.frequencyButtonsTemplate:i}

      <donation-form-section
        sectionBadge="${this.amountSelectionSectionNumber}"
        headline=${e}
        badgeMode=${this.formSectionNumberMode}
      >
        ${this.amountTitleDisplayMode===`slot`?a`<slot name="edit-donation-amount-title"></slot>`:i}
        <ul class="amount-selector">
          ${this.presetAmountsTemplate}
          ${this.customAmountMode===`display`?a`<li class="custom-amount">${this.customAmountTemplate}</li>`:i}
        </ul>

        <div class="errors">${this.error}</div>

        ${this.coverFeesCheckboxMode===`display`?a` <div class="checkbox-options">
              ${this.coverFeesCheckboxTemplate}
              ${this.frequencySelectionMode===W.Checkbox?this.frequencyCheckboxTemplate:i}
            </div>`:i}
      </donation-form-section>
    `}updated(e){e.has(`customAmountSelected`)&&this.customAmountButton&&(this.customAmountButton.checked=this.customAmountSelected),e.has(`amountOptions`)&&(this.customAmountSelected=!1,this.updateSelectedDonationInfo(),this.setupAmountColumnsLayoutConfig()),e.has(`amountSelectionLayout`)&&this.setupAmountColumnsLayoutConfig(),e.has(`donationInfo`)&&this.updateSelectedDonationInfo(),e.has(`defaultSelectedAmount`)&&this.defaultSelectedAmount!==void 0&&(this.customAmountSelected=!1,this.donationInfo=new b({donationType:this.donationInfo.donationType,amount:this.defaultSelectedAmount,coverFees:this.donationInfo.coverFees}))}get frequencyButtonsTemplate(){return a`
      <donation-form-section
        sectionBadge="1"
        headline="Choose a frequency"
        badgeMode=${this.formSectionNumberMode}
      >
        <ul class="frequency-selector">
          ${this.frequencyTemplate}
        </ul>
      </donation-form-section>
    `}get frequencyCheckboxTemplate(){return a`
      <div class="checkbox-option-container">
        <input
          type="checkbox"
          id="make-this-monthly"
          @input=${this.monthlyCheckboxChecked}
          .checked=${this.donationInfo.donationType===x.Monthly}
          tabindex="0"
        />
        <label for="make-this-monthly"> Make this monthly </label>
      </div>
    `}get coverFeesCheckboxTemplate(){return a`
      <div class="checkbox-option-container">
        <input
          type="checkbox"
          id="cover-fees"
          @input=${this.coverFeesChecked}
          .checked=${this.donationInfo.coverFees}
          tabindex="0"
        />
        <label for="cover-fees"> ${this.coverFeesTextTemplate} </label>
      </div>
    `}get amountSelectionSectionNumber(){return this.frequencySelectionMode===W.Button?2:1}get formSectionNumberMode(){switch(this.stepNumberMode){case V.ShowNumbers:return z.ShowBadge;case V.HideNumbers:return z.HideBadge}}setupAmountColumnsLayoutConfig(){let e=this.customAmountMode===`hide`&&this.coverFeesCheckboxMode===`hide`&&this.frequencySelectionMode===W.Hide,t=this.amountOptions.length,n=5,r=3;switch(t){case 7:n=5,r=3;break;case 6:n=4,r=2;break;case 5:n=4,r=3;break;case 4:if(e){n=4,r=0;break}n=3,r=2;break;case 3:n=2,r=1;break}this.amountSelectionLayout===G.SingleLine&&(n=t+3,r=3),this.style.setProperty(`--paymentSelectorAmountColumnCount`,`${n}`),this.style.setProperty(`--paymentSelectorCustomAmountColSpan`,`${r}`)}updateSelectedDonationInfo(){let e=this.shadowRoot?.activeElement===this.customAmountInput;if(!this.isCustomAmount&&!(this.customAmountSelected&&e)){let e=this.shadowRoot?.querySelector(`input[type="radio"][name="${H.Amount}"][value="${this.donationInfo.amount}"]`);e.checked=!0,this.customAmountSelected=!1,this.error=void 0,this.customAmountInput&&(this.customAmountInput.value=``)}else if(this.customAmountSelected=!0,!e){this.customAmountInput.value=this.customAmountDisplayValue;let e=this.getDonationInfoStatus(this.donationInfo.amount);this.handleDonationInfoStatus(e)}}get coverFeesTextTemplate(){return a` I'll generously add ${(0,D.default)(this.donationInfo.fee,{symbol:`$`}).format()} to cover fees. `}formatShortenedAmount(e){return(0,D.default)(e,{symbol:`$`,precision:e%1==0?0:2}).format()}get frequencyTemplate(){return a`
      <li>
        ${this.getRadioButton({group:H.DonationType,value:x.OneTime,displayText:`One time`,checked:this.donationInfo.donationType===x.OneTime})}
      </li>

      <li>
        ${this.getRadioButton({group:H.DonationType,value:x.Monthly,displayText:`Monthly`,checked:this.donationInfo.donationType===x.Monthly})}
      </li>
    `}get presetAmountsTemplate(){return a`
      ${this.amountOptions.map(e=>{let t=!this.customAmountSelected&&e===this.donationInfo.amount,n=this.formatShortenedAmount(e);return a`
          <li>
            ${this.getRadioButton({group:H.Amount,value:`${e}`,displayText:`${n}`,checked:t})}
          </li>
        `})}
    `}getRadioButton(e){let t=`${e.group}-${e.value}-option`;return a`
      <div class="selection-button">
        <input
          type="radio"
          name=${e.group}
          value=${e.value}
          id=${t}
          tabindex="0"
          .checked=${e.checked}
          @change=${this.radioSelected}
          @click=${t=>{e.group===H.Amount&&parseFloat(e.value)===this.donationInfo.amount&&this.radioSelected(t)}}
        />
        <label for=${t}> ${e.displayText} </label>
      </div>
    `}get isCustomAmount(){return!this.amountOptions.includes(this.donationInfo.amount)}get customAmountDisplayValue(){return this.isCustomAmount?(0,D.default)(this.donationInfo.amount,{symbol:``,separator:``}).format():``}get customAmountTemplate(){return a`
      <div class="selection-button">
        <input
          type="radio"
          name=${H.Amount}
          value="custom"
          id="custom-amount-button"
          tabindex="0"
          @change=${this.customRadioSelected}
        />

        <label for="custom-amount-button">
          <span class="custom-amount-text">Custom: $</span
          ><input
            type="text"
            id="custom-amount-input"
            tabindex="-1"
            value=${this.customAmountDisplayValue}
            @input=${this.customAmountChanged}
            @keydown=${this.currencyValidator.keydown}
            @focus=${this.customAmountFocused}
            @blur=${this.customAmountBlurred}
          />
        </label>
      </div>
    `}customRadioSelected(){this.customAmountInput.focus()}customAmountFocused(e){let t=e.target;this.customAmountSelected=!0,this.handleCustomAmountInput(t.value)}customAmountBlurred(){this.updateSelectedDonationInfo()}coverFeesChecked(e){let t=e.target.checked;this.updateDonationInfo({coverFees:t})}customAmountChanged(e){let t=e.target.value;this.customAmountSelected=!0,this.handleCustomAmountInput(t)}handleCustomAmountInput(e){let t=Re(e);isNaN(t)?this.dispatchEditDonationError(U.InvalidDonationAmount):this.amountChanged(t)}handleDonationInfoStatus(e){switch(e){case U.ValidDonationAmount:this.error=void 0;break;case U.DonationTooHigh:this.error=a`
          To make a donation of $10,000 or more, please contact our philanthropy
          department at
          <a href="mailto:donations@archive.org">donations@archive.org</a>
        `,this.dispatchEditDonationError(e);break;case U.DonationTooLow:this.customAmountInput.value.length>0&&(this.error=a` Please select an amount (minimum $1) `),this.dispatchEditDonationError(e);break;case U.InvalidDonationAmount:this.error=a` Please enter a valid donation amount `,this.dispatchEditDonationError(e);break}}amountChanged(e){let t=this.getDonationInfoStatus(e);this.handleDonationInfoStatus(t),t===U.ValidDonationAmount&&this.updateDonationInfo({amount:e})}getDonationInfoStatus(e){return isNaN(e)?U.InvalidDonationAmount:e>=1e4?U.DonationTooHigh:e<1?U.DonationTooLow:U.ValidDonationAmount}radioSelected(e){let t=e.target,n=t.name,{value:r}=t;switch(n){case H.Amount:this.presetAmountChanged(parseFloat(r));break;case H.DonationType:this.updateDonationInfo({donationType:r});break;default:break}}monthlyCheckboxChecked(e){let t=e.target.checked?x.Monthly:x.OneTime;this.updateDonationInfo({donationType:t})}dispatchEditDonationError(e){let t=new CustomEvent(`editDonationError`,{detail:{error:e}});this.dispatchEvent(t)}presetAmountChanged(e){this.error=void 0,this.customAmountSelected=!1,this.customAmountInput&&(this.customAmountInput.value=``),this.updateDonationInfo({amount:e})}updateDonationInfo(e){let t=new b({donationType:e.donationType??this.donationInfo.donationType,amount:e.amount??this.donationInfo.amount,coverFees:e.coverFees??this.donationInfo.coverFees});this.donationInfo=t;let n=new CustomEvent(`donationInfoChanged`,{detail:{donationInfo:t}});this.dispatchEvent(n)}static get styles(){let e=n`var(--paymentButtonBorderColor, #333)`,t=n`var(--paymentButtonGridGap, 1rem)`,r=n`var(--paymentButtonFontSize, 1.6rem)`,i=n`var(--paymentButtonFontColor, #000)`,a=n`var(--paymentButtonSelectedFontColor, #000)`,o=n`var(--paymentButtonSelectedColor, #f9bf3b)`,s=n`var(--paymentButtonFocusedOutlineColor, #7fb3f9)`,c=n`var(--paymentButtonColor, #fff)`,l=n`var(--coverFeesFontSize, 1.2rem)`,u=n`var(--coverFeesFontWeight, bold)`,d=n`var(--customAmountWidth, 4rem)`,f=n`var(--inputFieldFontColor, #333)`,p=n`var(--inputBorder, 1px solid #d9d9d9)`;return n`
      :host {
        --formSectionContentBackgroundColor: var(
          --editFormBgColor,
          transparent
        );
        --formSectionBadgeBackgroundColor: var(--editFormBadgeBgColor, #333);
        --formSectionBadgeFontColor: var(--editFormBadgeFontColor, #fff);
        --formSectionTextColor: var(--editFormTextColor, #333);
      }
      .errors {
        color: red;
        font-size: 1.4rem;
        margin-top: 0.5rem;
      }

      ul {
        list-style: none;
        margin: 0;
        padding: 0;
        display: grid;
        grid-gap: ${t};
      }

      li {
        margin: 0;
        padding: 0;
        display: inline-block;
      }

      .frequency-selector {
        grid-template-columns: repeat(2, 1fr);
      }

      .amount-selector {
        grid-template-columns: repeat(${n`var(--paymentSelectorAmountColumnCount, 5)`}, 1fr);
      }

      .custom-amount {
        grid-column: span ${n`var(--paymentSelectorCustomAmountColSpan, 3)`};
      }

      .selection-button {
        height: 3rem;
      }

      .selection-button label {
        padding: 0 0.3rem;
        display: flex;
        cursor: pointer;
        text-align: center;
        font-size: ${r};
        font-weight: bold;
        border: 1px solid ${e};
        border-radius: 5px;
        background-color: #ccc;
        height: 100%;
        justify-content: center;
        align-items: center;
      }

      label[for='custom-amount-button'] {
        padding: 0;
        display: flex;
        align-items: center;
        justify-content: center;
      }

      .custom-amount-text {
        white-space: nowrap;
        margin-right: 0.5rem;
      }

      input[type='radio'] {
        opacity: 0;
        width: 0;
        height: 0;
        position: absolute;
      }

      input[type='radio'] + label {
        color: ${i};
        background-color: ${c};
      }

      input[type='radio']:checked + label {
        color: ${a};
        background-color: ${o};
      }

      input[type='radio']:focus + label {
        outline: 2px solid ${s};
      }

      .checkbox-options {
        margin-top: 1rem;
      }

      .checkbox-option-container {
        display: flex;
        align-items: center;
        justify-content: center;
      }

      .checkbox-option-container input {
        width: 2rem;
      }

      .checkbox-option-container label {
        font-size: ${l};
        font-weight: ${u};
        flex: 1;
      }

      #custom-amount-input {
        width: ${d};
        font-size: 1.6rem;
        font-weight: bold;
        color: ${f};
        padding: 0.1rem;
        border: ${p};
        appearance: none;
        -moz-appearance: none;
        -webkit-appearance: none;
      }
    `}};p([r({type:Object})],K.prototype,`donationInfo`,void 0),p([r({type:String})],K.prototype,`stepNumberMode`,void 0),p([r({type:Number})],K.prototype,`defaultSelectedAmount`,void 0),p([r({type:Array})],K.prototype,`amountOptions`,void 0),p([r({type:String})],K.prototype,`amountSelectionLayout`,void 0),p([r({type:String,reflect:!0})],K.prototype,`frequencySelectionMode`,void 0),p([r({type:String,reflect:!0})],K.prototype,`customAmountMode`,void 0),p([r({type:String,reflect:!0})],K.prototype,`coverFeesCheckboxMode`,void 0),p([r({type:String,reflect:!0})],K.prototype,`amountTitleDisplayMode`,void 0),p([s()],K.prototype,`error`,void 0),p([s()],K.prototype,`customAmountSelected`,void 0),p([d(`#custom-amount-button`)],K.prototype,`customAmountButton`,void 0),p([d(`#custom-amount-input`)],K.prototype,`customAmountInput`,void 0),K=p([u(`donation-form-edit-donation`)],K);var q=class extends c{constructor(){super(...arguments),this.analyticsCategory=`DonationForm`,this.amountOptions=S,this.donationInfo=C,this.amountSelectionLayout=G.MultiLine,this.frequencySelectionMode=W.Button,this.donorEmail=``,this.lazyLoaderService=new ee,this.recaptchaManagerSetup=!1}updated(e){var t,n,r;e.has(`referrer`)&&this.referrer&&((t=this.braintreeManager)==null||t.setReferrer(this.referrer),this.logDonationFlowEvent(`referrer`,this.referrer)),e.has(`loggedInUser`)&&this.loggedInUser&&((n=this.braintreeManager)==null||n.setLoggedInUser(this.loggedInUser)),e.has(`origin`)&&this.origin&&((r=this.braintreeManager)==null||r.setOrigin(this.origin),this.logDonationFlowEvent(`origin`,this.origin)),(e.has(`paymentClients`)||e.has(`braintreeAuthToken`)||e.has(`endpointManager`)||e.has(`environment`))&&(this.setupBraintreeManager(),this.setupRecaptchaManager()),e.has(`recaptchaSiteKey`)&&this.setupRecaptchaManager(),(e.has(`braintreeManager`)||e.has(`recaptchaManager`)||e.has(`modalManager`)||e.has(`recaptchaElement`))&&this.setupPaymentFlowHandlers(),(e.has(`environment`)||e.has(`lazyLoaderService`))&&this.environment&&(this.paymentClients=new E(this.lazyLoaderService,this.environment))}showConfirmationStepDev(e){return f(this,void 0,void 0,function*(){this.donationForm.showConfirmationModalDev(e)})}showUpsellModalDev(e){return f(this,void 0,void 0,function*(){this.donationForm.showUpsellModalDev(e)})}setupBraintreeManager(){this.braintreeManager===void 0&&this.braintreeAuthToken&&this.endpointManager&&this.paymentClients&&this.environment&&(this.braintreeManager=new be({paymentClients:this.paymentClients,endpointManager:this.endpointManager,authorizationToken:this.braintreeAuthToken,venmoProfileId:this.venmoProfileId,googlePayMerchantId:this.googlePayMerchantId,hostedFieldConfig:this.hostedFieldConfig,hostingEnvironment:this.environment,referrer:this.referrer,loggedInUser:this.loggedInUser,origin:this.origin}),this.braintreeManager.on(`paymentProvidersHostedFieldsRetry`,e=>{let t=new CustomEvent(`paymentProvidersHostedFieldsRetry`,{detail:{retryNumber:e}});this.dispatchEvent(t)}),this.braintreeManager.on(`paymentProvidersHostedFieldsFailed`,e=>{let t=new CustomEvent(`paymentProvidersHostedFieldsFailed`,{detail:{error:e}});this.dispatchEvent(t)}))}setupRecaptchaManager(){return f(this,void 0,void 0,function*(){if(!this.recaptchaSiteKey||!this.paymentClients||this.recaptchaManagerSetup)return;this.recaptchaManagerSetup=!0;let e=yield this.paymentClients.recaptchaLibrary.get();this.recaptchaManager=new Pe({grecaptchaLibrary:e,siteKey:this.recaptchaSiteKey})})}firstUpdated(){this.configureFromQueryParams(),this.trackViewedEvent()}configureFromQueryParams(){let e=new URLSearchParams(window.location.search),t=this.amountOptions,n=e.get(`dollarAmounts`);n&&(t=n.slice(1,-1).split(`,`).map(e=>parseFloat(e)).filter(e=>!isNaN(e)));let r=this.donationInfo.coverFees,i=e.get(`coverFees`);i&&(r=i===`true`);let a=this.donationInfo.donationType;e.get(`contrib_type`)===`monthly`&&(a=x.Monthly);let o=this.donationInfo.amount,s=e.get(`amt`);if(s){let e=T(s.replace(/usd|\$/gi,``).trim());e>0&&(o=e)}let c=e.get(`amountLayout`);if(c){let e=c;Object.values(G).includes(e)&&(this.amountSelectionLayout=e)}let l=e.get(`frequencyMode`);if(l){let e=l;Object.values(W).includes(e)&&(this.frequencySelectionMode=e)}let u=new b({donationType:a,amount:o,coverFees:r});this.amountOptions=t,this.donationInfo=u}setupPaymentFlowHandlers(){var e;this.paymentFlowHandlers||!this.braintreeManager||!this.recaptchaManager||!this.modalManager||!this.recaptchaElement||(this.paymentFlowHandlers=new Ne({braintreeManager:this.braintreeManager,modalManager:this.modalManager,recaptchaManager:this.recaptchaManager,resources:{analytics:{logEvent:this.logEvent.bind(this),logDonationFlowEvent:this.logDonationFlowEvent.bind(this)}}}),this.donationForm.braintreeManager=this.braintreeManager,this.donationForm.paymentFlowHandlers=this.paymentFlowHandlers,this.braintreeManager.startup(),(e=this.paymentFlowHandlers)==null||e.startup(),this.recaptchaManager.setup(this.recaptchaElement,1,`light`,`image`))}get hostedFieldConfig(){let e={input:{"font-size":`16px`,"font-family":`"Helvetica Neue", Helvetica, Arial, sans-serif`,"font-weight":`700`,color:`#333`},":focus":{color:`#333`},".valid":{color:`#333`},".invalid":{color:`#333`}},t={number:{selector:`#braintree-creditcard`},cvv:{selector:`#braintree-cvv`},expirationDate:{selector:`#braintree-expiration`}},n=this.creditCardFieldsElement.hostedFieldContainer;return new Fe({hostedFieldStyle:e,hostedFieldFieldOptions:t,hostedFieldContainer:n})}render(){return a`
      <div class="donation-form-controller-container">
        <donation-form
          .environment=${this.environment}
          .braintreeManager=${this.braintreeManager}
          .contactForm=${this.contactForm}
          .amountOptions=${this.amountOptions}
          .donationInfo=${this.donationInfo}
          .amountSelectionLayout=${this.amountSelectionLayout}
          .frequencySelectionMode=${this.frequencySelectionMode}
          @donationInfoChanged=${this.donationInfoChanged}
          @paymentProviderSelected=${this.paymentProviderSelected}
          @paymentFlowStarted=${this.paymentFlowStarted}
          @paymentFlowConfirmed=${this.paymentFlowConfirmed}
          @paymentFlowCancelled=${this.paymentFlowCancelled}
          @paymentFlowError=${this.paymentFlowError}
        >
          <!--
            Why are these slots here?

            Due to the way Braintree, PayPal, and Recaptcha work, they cannot exist
            in the shadowDOM so must exist in the clearDOM and get passed
            in through a <slot>.

            Braintree / PayPal are working on a solution to this. See:
            - https://github.com/braintree/braintree-web-drop-in/issues/614#issuecomment-616796104
            - https://github.com/braintree/braintree-web-drop-in/issues/296#issuecomment-616749307
            - https://github.com/paypal/paypal-checkout-components/issues/353#issuecomment-595956216
          -->
          <div slot="braintree-hosted-fields">
            <credit-card-fields></credit-card-fields>
          </div>

          <!--
            Form autocompletion does not work in the shadowDOM so
            we slot the contact form in from the lightDOM and pass
            in a reference to it in the <donation-form> tag above
          -->
          <div slot="contact-form">
            <contact-form .donorEmail=${this.donorEmail}></contact-form>
          </div>

          <div slot="paypal-button">
            <div id="paypal-button"></div>
          </div>

          <slot name="recaptcha" slot="recaptcha"> </slot>
        </donation-form>
      </div>

      ${this.getStyles}
    `}createRenderRoot(){return this}donationInfoChanged(e){this.logEvent(`DonationInfoChanged`),this.donationInfo=e.detail.donationInfo}trackViewedEvent(){this.logEvent(`Viewed`)}paymentProviderSelected(e){let t=e.detail.paymentProvider,n=e.detail.previousPaymentProvider,r=this.removeSpaces(t??`unset`),i=`ProviderFirstSelected-${r}`,a;n!==void 0&&(i=`ProviderChangedTo-${r}`,a=`ProviderChangedFrom-${this.removeSpaces(n)}`),this.logEvent(i,a)}paymentFlowConfirmed(e){let t=e.detail.paymentProvider,n=this.removeSpaces(t);this.logEvent(`PaymentFlowConfirmed`,n)}paymentFlowStarted(e){let t=e.detail.paymentProvider,n=this.removeSpaces(t);this.logEvent(`PaymentFlowStarted`,n)}paymentFlowCancelled(e){let t=e.detail.paymentProvider,n=this.removeSpaces(t);this.logEvent(`PaymentFlowCancelled`,n)}paymentFlowError(e){let t=e.detail.paymentProvider,n=`${this.removeSpaces(t)}-${e.detail.error}`;this.logEvent(`PaymentFlowError`,n)}removeSpaces(e){return e.replace(/\s+/g,``)}logEvent(e,t){var n;let r={action:e,label:t,category:this.analyticsCategory};(n=this.analyticsHandler)==null||n.sendEvent(r)}logDonationFlowEvent(e,t){var n;let r={action:e,label:t,category:`DonationFlow`};(n=this.analyticsHandler)==null||n.sendEventNoSampling(r)}get getStyles(){return a`
      <style>
        .donation-form-controller-container {
          color: var(--donateFormTextColor, #333);
          background-color: var(--donateFormBgColor, transparent);

          --formSectionContentBackgroundColor: var(--donateFormBgColor, transparent);

          --editFormBadgeBgColor: var(--donateFormBadgeBgColor, #333);
          --formSectionBadgeBackgroundColor: var(--donateFormBadgeBgColor, #333);

          --editFormBadgeFontColor: var(--donateFormBadgeTextColor, #fff);
          --formSectionBadgeFontColor: var(--donateFormBadgeTextColor, #fff);

          --paymentButtonFontColor: var(--donateFormPaymentOptionTextColor);
          --paymentButtonColor: var(--donateFormPaymentOptionBgColor);

          --paymentButtonSelectedColor: var(--donateFormSelectedOptionBgColor);
          --paymentButtonSelectedFontColor: var(--donateFormSelectedOptionTextColor);
        }
        .donation-form-controller-container donation-form:focus {
          outline: none;
        }

        .donation-form-controller-container #paypal-button {
          opacity: 0.001;
          width: 5rem;
          height: 3rem;
          overflow: hidden;
        }

        .donation-form-controller-container div[slot='braintree-hosted-fields'] {
          background-color: white;
        }
      </style>
    `}};p([r({type:String})],q.prototype,`environment`,void 0),p([r({type:String})],q.prototype,`braintreeAuthToken`,void 0),p([r({type:String})],q.prototype,`recaptchaSiteKey`,void 0),p([r({type:String})],q.prototype,`venmoProfileId`,void 0),p([r({type:String})],q.prototype,`googlePayMerchantId`,void 0),p([r({type:String})],q.prototype,`analyticsCategory`,void 0),p([r({type:Array})],q.prototype,`amountOptions`,void 0),p([r({type:Object})],q.prototype,`donationInfo`,void 0),p([r({type:String})],q.prototype,`amountSelectionLayout`,void 0),p([r({type:String})],q.prototype,`frequencySelectionMode`,void 0),p([r({type:String})],q.prototype,`referrer`,void 0),p([r({type:String})],q.prototype,`loggedInUser`,void 0),p([r({type:String})],q.prototype,`origin`,void 0),p([r({type:String})],q.prototype,`donorEmail`,void 0),p([r({type:Object})],q.prototype,`endpointManager`,void 0),p([r({type:Object})],q.prototype,`analyticsHandler`,void 0),p([r({type:Object})],q.prototype,`modalManager`,void 0),p([r({type:Object})],q.prototype,`recaptchaElement`,void 0),p([r({type:Object})],q.prototype,`braintreeManager`,void 0),p([r({type:Object})],q.prototype,`recaptchaManager`,void 0),p([r({type:Object})],q.prototype,`paymentFlowHandlers`,void 0),p([r({type:Object})],q.prototype,`paymentClients`,void 0),p([r({type:Object})],q.prototype,`lazyLoaderService`,void 0),p([d(`donation-form`)],q.prototype,`donationForm`,void 0),p([d(`credit-card-fields`)],q.prototype,`creditCardFieldsElement`,void 0),p([d(`contact-form`)],q.prototype,`contactForm`,void 0),q=p([u(`donation-form-controller`)],q);var ze=a`
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 468 300" aria-labelledby="applePayTitleID applePayDescID">
  <title id="donateTitleID">ApplePay icon</title>
  <desc id="donateDescID">An illustration of the Apple Pay logo</desc>
  <g fill="none">
    <path
      fill="#000"
      d="M425.540484,0 L41.8576242,0 C40.2596386,0 38.6588291,0 37.0636954,0.0093185241 C35.715305,0.0189194277 34.3700489,0.0338855422 33.0247929,0.0705948795 C30.0908697,0.149943524 27.1318148,0.323042169 24.2346009,0.84375 C21.2910768,1.37349398 18.5519955,2.2375753 15.8798381,3.59807982 C13.2528614,4.93401732 10.8478351,6.68138178 8.76362011,8.76647214 C6.67850151,10.8515625 4.93113705,13.2526355 3.59548193,15.882436 C2.23469503,18.5545934 1.37004895,21.2945218 0.843975904,24.240305 C0.320472515,27.138366 0.146216114,30.0968562 0.0671498494,33.0276732 C0.0310052711,34.3729292 0.0155026355,35.7181852 0.00672063253,37.0631589 C1.12951807e-05,38.6617093 1.12951807e-05,40.2591303 1.12951807e-05,41.8605045 L1.12951807e-05,257.36634 C1.12951807e-05,258.967715 1.12951807e-05,260.56234 0.00672063253,262.163997 C0.0155026355,263.50897 0.0310052711,264.854226 0.0671498494,266.199482 C0.146216114,269.12773 0.320472515,272.08622 0.843975904,274.983434 C1.37004895,277.930346 2.23469503,280.669145 3.59548193,283.341585 C4.93113705,285.971386 6.67850151,288.375565 8.76362011,290.457549 C10.8478351,292.546028 13.2528614,294.29311 15.8798381,295.625659 C18.5519955,296.989834 21.2910768,297.854198 24.2346009,298.38366 C27.1318148,298.900979 30.0908697,299.077184 33.0247929,299.156532 C34.3700489,299.187029 35.715305,299.205102 37.0636954,299.211314 C38.6588291,299.223739 40.2596386,299.223739 41.8576242,299.223739 L425.540484,299.223739 C427.135646,299.223739 428.736483,299.223739 430.331306,299.211314 C431.676591,299.205102 433.021875,299.187029 434.373287,299.156532 C437.300998,299.077184 440.259488,298.900979 443.16379,298.38366 C446.10336,297.854198 448.843006,296.989834 451.515446,295.625659 C454.145247,294.29311 456.543213,292.546028 458.631721,290.457549 C460.713422,288.375565 462.460759,285.971386 463.799831,283.341585 C465.163695,280.669145 466.027494,277.930346 466.551026,274.983434 C467.07484,272.08622 467.244832,269.12773 467.324181,266.199482 C467.36089,264.854226 467.378991,263.50897 467.385203,262.163997 C467.3976,260.56234 467.3976,258.967743 467.3976,257.36634 L467.3976,41.8605045 C467.3976,40.2591303 467.3976,38.6617093 467.385203,37.0631589 C467.378991,35.7181852 467.36089,34.3729292 467.324181,33.0276732 C467.244804,30.0968562 467.07484,27.138366 466.551026,24.240305 C466.027523,21.2945218 465.163695,18.5545934 463.799831,15.882436 C462.460759,13.2526355 460.713422,10.8515625 458.631721,8.76647214 C456.543213,6.68138178 454.145247,4.93401732 451.515446,3.59807982 C448.843006,2.2375753 446.10336,1.37349398 443.16379,0.84375 C440.259516,0.323042169 437.301026,0.149943524 434.373287,0.0705948795 C433.021875,0.0338855422 431.676591,0.0189194277 430.331306,0.0093185241 C428.736483,0 427.135646,0 425.540484,0 L425.540484,0 Z"
    />
    <path
      fill="#FFF"
      d="M425.540484,9.97364458 L430.260429,9.98268072 C431.539044,9.99171687 432.817686,10.0055535 434.103389,10.0405685 C436.339863,10.1009977 438.956052,10.2221386 441.394682,10.659262 C443.514505,11.0410392 445.292338,11.6216114 446.998503,12.4902108 C448.682897,13.3461032 450.22613,14.4677146 451.573588,15.813253 C452.926186,17.1678276 454.049238,18.713573 454.916425,20.4166039 C455.780196,22.1094691 456.357097,23.8788592 456.736615,26.0142131 C457.172609,28.4262989 457.293185,31.0496047 457.35449,33.299887 C457.388912,34.5700301 457.405855,35.8401732 457.412095,37.1405309 C457.423956,38.7131024 457.423956,40.2848268 457.423956,41.8605045 L457.423956,257.36634 C457.423956,258.942018 457.423956,260.510919 457.411813,262.117093 C457.405855,263.386954 457.38894,264.657097 457.354207,265.929499 C457.293185,268.176393 457.172637,270.798287 456.731278,273.238893 C456.357097,275.34488 455.780506,277.114552 454.911935,278.815889 C454.047261,280.514684 452.925932,282.058735 451.579236,283.404838 C450.224125,284.76026 448.685975,285.878483 446.981561,286.742282 C445.288131,287.606645 443.513347,288.186681 441.41442,288.564477 C438.926402,289.007812 436.200593,289.1298 434.147694,289.185429 C432.85609,289.214514 431.570698,289.231994 430.253991,289.238234 C428.68478,289.250095 427.109977,289.250095 425.540512,289.250095 L41.8576242,289.250095 C41.8367282,289.250095 41.8163968,289.250095 41.7952184,289.250095 C40.2438253,289.250095 38.6893261,289.250095 37.109695,289.237952 C35.821762,289.231994 34.5366529,289.214797 33.294183,289.186559 C31.1944089,289.1298 28.4669051,289.008095 25.9991905,288.5673 C23.8822195,288.186653 22.107436,287.606645 20.391698,286.730986 C18.7033791,285.874812 17.1663592,284.757718 15.8106551,283.399755 C14.4653991,282.056758 13.3474586,280.517508 12.4830949,278.816171 C11.617884,277.116529 11.039006,275.341491 10.6583867,273.210373 C10.2184111,270.774567 10.0975527,268.163121 10.0371517,265.931476 C10.0026732,264.653709 9.98827184,263.376224 9.98008283,262.106081 L9.97387048,258.356363 L9.97387048,257.36634 L9.97387048,41.8605045 L9.97387048,40.8704819 L9.97980045,37.1286709 C9.98827184,35.8509036 10.0026732,34.5734187 10.0371517,33.2967809 C10.0975527,31.0628765 10.2184111,28.4503012 10.6620294,25.9941642 C11.0392884,23.8822477 11.617884,22.1072101 12.487613,20.3990964 C13.3451995,18.7107492 14.4651167,17.1695218 15.8174605,15.8174887 C17.1644108,14.4699736 18.7064571,13.3497741 20.4055346,12.4856928 C22.1029179,11.6213291 23.8810617,11.0410392 25.998061,10.6601092 C28.4372553,10.2218562 31.0551958,10.1009977 33.2972892,10.0402861 C34.5753389,10.0055535 35.8533886,9.99171687 37.1218373,9.9829631 L41.8576242,9.97364458 L425.540484,9.97364458"
    />
    <g fill="#000">
      <path
        d="M64.3701386 18.7514966C68.3721341 13.7458678 71.0878627 7.02478351 70.371607.156635919 64.5132486.44793863 57.3642463 4.02159262 53.225325 9.03114646 49.5090129 13.3210561 46.2197715 20.3235599 47.0772734 26.9037933 53.6535818 27.4742282 60.223819 23.6166698 64.3701386 18.7514966M70.2968894 28.1885919C60.7465035 27.6197101 52.6263416 33.608895 48.0655453 33.608895 43.5022358 33.608895 36.5181714 28.4752636 28.9643216 28.6136295 19.1325163 28.7580384 10.0097377 34.3170181 5.02051515 43.1584055-5.24146716 60.845529 2.31238262 87.0817206 12.2916183 101.486888 17.1377591 108.613582 22.9781865 116.460599 30.6738472 116.178727 37.9449504 115.893411 40.7949222 111.470557 49.6332599 111.470557 58.4651029 111.470557 61.032328 116.178727 68.7292313 116.036098 76.7111119 115.893383 81.7012098 108.905845 86.5472941 101.772204 92.1067256 93.6481457 94.3825633 85.8036992 94.5257015 85.3730704 94.3825351 85.2304405 79.13438 79.3808641 78.9929926 61.8407474 78.8490355 47.1541604 90.9645005 40.1684017 91.5347094 39.7351751 84.6929926 29.6162462 74.0029511 28.4752636 70.2968894 28.1885919"
        transform="translate(63.226 81.89)"
      />
      <path
        d="M40.5024334.459215539C61.2600389.459215539 75.7143122 14.7676399 75.7143122 35.5995958 75.7143122 56.5059305 60.9626085 70.8886771 39.9819233 70.8886771L16.9992802 70.8886771 16.9992802 107.437566.394319742 107.437566.394319742.459215539 40.5024334.459215539 40.5024334.459215539zM16.999252 56.95065L36.0523017 56.95065C50.509427 56.95065 58.7375426 49.1672539 58.7375426 35.6739463 58.7375426 22.1820789 50.509427 14.4716214 36.1266522 14.4716214L16.999252 14.4716214 16.999252 56.95065 16.999252 56.95065zM80.0528476 85.271452C80.0528476 71.6294151 90.5060291 63.2525984 109.041449 62.2144301L130.391063 60.9546222 130.391063 54.9501323C130.391063 46.2758571 124.533862 41.0864557 114.74992 41.0864557 105.480784 41.0864557 99.6979342 45.5337073 98.2908371 52.5034268L83.1673243 52.5034268C84.0567915 38.4166704 96.0657712 28.0378676 115.341929 28.0378676 134.246278 28.0378676 146.329608 38.0463014 146.329608 53.6888842L146.329608 107.437595 130.983072 107.437595 130.983072 94.6120864 130.614143 94.6120864C126.092541 103.286362 116.231396 108.771753 106.001295 108.771753 90.7290807 108.771753 80.0528476 99.2823895 80.0528476 85.271452zM130.391063 78.2287939L130.391063 72.075603 111.189256 73.2610604C101.625542 73.9288597 96.2145005 78.1544433 96.2145005 84.8267325 96.2145005 91.6463108 101.848621 96.0949742 110.448546 96.0949742 121.642437 96.0949742 130.391063 88.3845167 130.391063 78.2287939zM160.81819 136.128737L160.81819 123.154528C162.002208 123.450546 164.670553 123.450546 166.006151 123.450546 173.419179 123.450546 177.423122 120.337453 179.868388 112.331006 179.868388 112.182277 181.278337 107.586324 181.278337 107.511973L153.107705 29.4464049 170.453375 29.4464049 190.175664 92.9075589 190.470242 92.9075589 210.19256 29.4464049 227.09495 29.4464049 197.883269 111.514477C191.21386 130.420266 183.503375 136.499106 167.34175 136.499106 166.00618 136.499106 162.002236 136.350377 160.81819 136.128737z"
        transform="translate(63.226 81.89) translate(112.952 7.853)"
      />
    </g>
  </g>
</svg>
`,Be=a`
<svg viewBox="0 0 469 300" xmlns="http://www.w3.org/2000/svg" aria-labelledby="googlePayTitleID googlePayDescID">
  <title id="googlePayTitleID">GooglePay icon</title>
  <desc id="googlePayDescID">The GooglePay logo</desc>
  <g fill="none">
    <path
      d="m426.541557.09900471h-384.5859537c-1.6020287 0-3.2068878 0-4.8060861.00848612-1.3501196.01131483-2.6974088.02545836-4.0475283.06223154-2.9408266.07920377-5.9071271.25175484-8.811158.77506548-2.9493177.52896804-5.6948441 1.39455212-8.3724397 2.75798848-2.6351391 1.33797799-5.043843 3.08894709-7.13553773 5.1765322-2.08886428 2.09041387-3.84090626 4.49481397-5.17970411 7.13116817-1.36427179 2.6759559-2.23038625 5.4198009-2.75684798 8.3701415-.52646173 2.9050811-.69911853 5.8667364-.77837083 8.8029334-.03679572 1.3492928-.05377835 2.695757-.06226967 4.042221-.00611375 1.6010478-.00611375 3.2020954-.00611375 4.8059718v215.8811936c0 1.603877 0 3.202097.00611375 4.805973.00849132 1.346464.02547395 2.695757.06226967 4.042221.0792523 2.933368.2519091 5.897852.77837083 8.800105.52646173 2.95034 1.39257619 5.694185 2.75684798 8.372969 1.33879785 2.633526 3.09083983 5.040754 5.17970411 7.12834 2.09169473 2.090414 4.50039863 3.841383 7.13553773 5.176532 2.6775956 1.366265 5.423122 2.23185 8.3724397 2.763646 2.9040309.517653 5.8703314.693033 8.811158.772236 1.3501195.031116 2.6974087.050918 4.0475283.056575 1.5991983.011315 3.2040574.011315 4.8060861.011315h384.5859537c1.599199 0 3.204058 0 4.803256-.011315 1.347289-.005657 2.694578-.025459 4.050359-.056575 2.935166-.079203 5.901466-.254583 8.811157-.772236 2.946488-.531796 5.692014-1.397381 8.37244-2.763646 2.635139-1.335149 5.038182-3.086118 7.132707-5.176532 2.086035-2.087586 3.838077-4.494814 5.179705-7.12834 1.367102-2.678784 2.233216-5.422629 2.756848-8.372969.526462-2.902253.696288-5.866737.77554-8.800105.036796-1.346464.053778-2.695757.06227-4.042221.011322-1.603876.011322-3.202096.011322-4.805973v-215.8811936c0-1.6038764 0-3.204924-.011322-4.8059718-.008492-1.346464-.025474-2.6929282-.06227-4.042221-.079252-2.936197-.249078-5.8978523-.77554-8.8029334-.523632-2.9503406-1.389746-5.6941856-2.756848-8.3701415-1.341628-2.6363542-3.09367-5.0407543-5.179705-7.13116817-2.094525-2.08758511-4.497568-3.83855421-7.132707-5.1765322-2.680426-1.36343636-5.425952-2.22902044-8.37244-2.75798848-2.909691-.52331064-5.875991-.69586171-8.811157-.77506548-1.355781-.03677318-2.70307-.05091671-4.050359-.06223154-1.599198-.00848612-3.204057-.00848612-4.803256-.00848612"
      fill="#3c4043"
    />
    <path
      d="m426.541557 10.0899948 4.732495.0084861c1.279359.0084861 2.561548.0226296 3.849398.0594028 2.241708.0594028 4.865525.1810372 7.311024.6194866 2.12283.3818754 3.906007.9645889 5.615592 1.8330016 1.689772.857098 3.235192 1.9829231 4.585313 3.3293871 1.35578 1.357779 2.482295 2.9050813 3.351239 4.610791.866115 1.6972237 1.443525 3.4679938 1.825633 5.6064956.435888 2.4185437.557598 5.0464117.617037 7.3008905.036796 1.270089.053778 2.5430068.059439 3.8470403.011322 1.5755893.011322 3.14835.011322 4.726768v215.8811936c0 1.578418 0 3.151179-.011322 4.757884-.005661 1.272918-.022643 2.545836-.059439 3.821582-.059439 2.248821-.181149 4.87669-.622697 7.320692-.376448 2.110214-.953858 3.880984-1.825633 5.586695-.866115 1.70288-1.989799 3.247354-3.339919 4.596647-1.358611 1.357779-2.89837 2.477946-4.607955 3.343531-1.698263.865584-3.47578 1.448298-5.578796 1.824515-2.496448.444107-5.227821.56857-7.285551.622315-1.293511.031116-2.581361.048088-3.903177.053746-1.570893.011314-3.150279.011314-4.724003.011314h-384.5859537c-.0198131 0-.0396261 0-.0622696 0-1.5539113 0-3.1134834 0-4.6956991-.011314-1.2906803-.005658-2.5785302-.02263-3.8239236-.050917-2.1058469-.056574-4.8400513-.181037-7.3138554-.622315-2.1228296-.379046-3.9003455-.96176-5.6212527-1.838659-1.6926027-.857098-3.2323618-1.977266-4.5909726-3.337874-1.3472892-1.343635-2.4681433-2.88528-3.3342577-4.590989-.8689448-1.702882-1.449185-3.479309-1.8312943-5.614982-.4387181-2.441174-.560427-5.054898-.6226967-7.292405-.0339652-1.278574-.0481174-2.559979-.0566087-3.830068l-.00566093-3.756521v-217.8641178l.00566093-3.7480355c.0084913-1.2814039.0226435-2.5599791.0566087-3.8385543.0622697-2.2403353.1839786-4.8568884.6283576-7.3178627.3764484-2.1130435.9566886-3.8922997 1.8284639-5.6036669.8604536-1.6915663 1.984138-3.2360398 3.337088-4.5881613 1.3501196-1.3521215 2.8983699-2.4722892 4.599464-3.3378733 1.7010941-.8684127 3.4842709-1.4482975 5.6071005-1.8301729 2.4454996-.4384494 5.0693169-.5600838 7.3166859-.6194866 1.2793586-.0367732 2.5615477-.0509167 3.8324149-.0594028l4.7466469-.0084861z"
      fill="#fffffe"
    />
    <g transform="translate(53.778 84.906)">
      <g fill="#3c4043" transform="translate(158.35 8.41)">
        <path
          d="m13.4306616 63.5773585v42.7212935h-13.34775628v-105.45768165h35.40057118c8.5392479-.16819407 16.8297798 3.1115903 22.8818681 9.16657685 12.1041765 11.4371967 12.8503244 30.6954178 1.4922957 43.0576819-.4974319.5045823-.9948639 1.0091644-1.4922957 1.5137466-6.217899 5.9708896-13.8451883 8.9983828-22.8818681 8.9983828zm0-49.7854447v36.8345013h22.3844362c4.9743191.1681941 9.7828276-1.8501348 13.1819457-5.4663073 6.9640468-7.316442 6.7982361-19.090027-.4145266-26.1541779-3.3991181-3.3638814-7.9589106-5.2140161-12.7674191-5.2140161zm85.3095733 17.9967654c9.8657331 0 17.6588331 2.6911052 23.3793001 7.9892184 5.720466 5.2981131 8.539248 12.6986522 8.539248 22.0334232v44.4873312h-12.767419v-10.0075469h-.580338c-5.554657 8.2415099-12.850324 12.3622639-22.0528145 12.3622639-7.7931 0-14.4255256-2.354717-19.6485607-7.064151-5.1401297-4.3730455-8.0418158-10.8485172-7.8760052-17.660377 0-7.4846361 2.8187808-13.3714286 8.3734372-17.8285714 5.5546563-4.4571429 13.0161351-6.6436658 22.3015308-6.6436658 7.9589104 0 14.4255254 1.5137466 19.5656554 4.3730458v-3.1115903c0-4.6253369-1.989728-8.9983828-5.471751-12.025876-3.564929-3.1956874-8.124721-4.9617251-12.8503246-4.9617251-7.4614787 0-13.3477564 3.1956874-17.658833 9.587062l-11.7725553-7.4846362c6.3008043-9.3347708 15.834916-14.0442048 28.5194298-14.0442048zm-17.2443063 52.3924529c0 3.5320754 1.6581063 6.8118598 4.3939818 8.8301887 2.9845915 2.3547169 6.6324255 3.6161724 10.3631649 3.5320754 5.6375617 0 11.0264077-2.2706199 15.0058627-6.3072776 4.393981-4.2048518 6.632426-9.1665768 6.632426-14.8851752-4.145267-3.3638815-9.948639-5.0458221-17.410117-4.961725-5.3888462 0-9.9486387 1.3455525-13.5964727 3.9525605-3.5649288 2.6070082-5.3888457 5.8867925-5.3888457 9.8393532z"
        />
        <path
          d="m203.947 34.143-44.603 103.86h-13.762l16.581-36.33-29.266-67.53h14.509l21.141 51.804h.248l20.644-51.804z"
        />
      </g>
      <path
        d="m117.808458 62.7363881c0-4.1207546-.331621-8.2415094-.994864-12.278167h-56.2927112v23.2948786h32.2501691c-1.3264851 7.4846362-5.6375617 14.2123989-11.938366 18.4172507v15.1374666h19.2340341c11.275123-10.5121296 17.741738-26.0700811 17.741738-44.5714289z"
        fill="#4285f4"
      />
      <path
        d="m60.5208828 121.940701c16.0836319 0 29.6801042-5.382211 39.5458372-14.632884l-19.2340341-15.1374666c-5.3888456 3.7002695-12.2699871 5.8026954-20.3118031 5.8026954-15.5862 0-28.7681456-10.6803234-33.4937488-24.9768194h-19.81437125v15.6420486c10.11444885 20.435579 30.75787325 33.302426 53.30812005 33.302426z"
        fill="#34a853"
      />
      <path
        d="m27.027134 72.9962264c-2.4871596-7.4846361-2.4871596-15.6420485 0-23.2107817v-15.5579515h-19.81437125c-8.53924786 17.0716981-8.53924786 37.2549866 0 54.3266847z"
        fill="#fbbc04"
      />
      <path
        d="m60.5208828 24.8086253c8.5392479-.168194 16.7468744 3.1115903 22.881868 9.0824798l17.0784962-17.3239891c-10.8605972-10.25983837-25.120312-15.89433972-39.9603642-15.72614565-22.5502468 0-43.1936712 12.95094345-53.30812005 33.38652285l19.81437125 15.6420486c4.7256032-14.380593 17.9075488-25.0609165 33.4937488-25.0609165z"
        fill="#ea4335"
      />
    </g>
  </g>
</svg>
`,Ve=a`
<svg viewBox="0 0 468 300" xmlns="http://www.w3.org/2000/svg" aria-labelledby="payPalTitleID payPalDescID">
  <title id="payPalTitleID">PayPal icon</title>
  <desc id="payPalDescID">The PayPal logo</desc>
  <g fill="none" fill-rule="nonzero">
    <g>
      <path
        d="m426.088936 0h-384.1773743c-1.6000453 0-3.202918 0-4.8001076.0093427-1.3501283.00962581-2.6971182.02463075-4.044108.06143532-2.9377047.0795545-5.9005735.2531022-8.8015214.77516088-2.9473179.53111826-5.6899295 1.39744123-8.365531 2.76147524-2.6303624 1.33940326-5.0384885 3.09130081-7.1253898 5.18180042-2.08780603 2.09049964-3.8374226 4.49780164-5.1747992 7.13442444-1.36254076 2.6790896-2.22830126 5.4261262-2.75505234 8.3795514-.52417811 2.9055793-.6986591 5.8717446-.77782727 8.8101648-.03619117 1.348746-.05171378 2.697492-.06050711 4.0459548-.00671798 1.6026975-.00671798 3.2042625-.00671798 4.8097911v216.0649119c0 1.605529 0 3.204291.00671798 4.810103.00879333 1.348462.02431594 2.697208.06050711 4.045954.07916817 2.935844.25364916 5.90201.77782727 8.80674.52675108 2.954557 1.39251158 5.700461 2.75505234 8.379834 1.3373766 2.636623 3.08699317 5.047039 5.1747992 7.134425 2.0869013 2.093897 4.4950274 3.845511 7.1253898 5.181517 2.6756015 1.367714 5.4182131 2.23432 8.365531 2.765156 2.9009479.518661 5.8638167.695323 8.8015214.774877 1.3469898.030576 2.6939797.048696 4.044108.054924 1.5971896.012457 3.2000623.012457 4.8001076.012457h384.1773743c1.597218 0 3.200119 0 4.796997-.012457 1.347018-.006228 2.694037-.024348 4.04719-.054924 2.931485-.079554 5.893788-.256216 8.801833-.774877 2.943359-.530836 5.686536-1.397442 8.36242-2.765156 2.63319-1.336006 5.034248-3.08762 7.125447-5.181517 2.084385-2.087386 3.833973-4.497802 5.174771-7.134425 1.365622-2.679373 2.230535-5.425277 2.754741-8.379834.524489-2.90473.694701-5.870896.774152-8.80674.036756-1.348746.05488-2.697492.061101-4.045954.012412-1.605812.012412-3.204546.012412-4.810103v-216.0649119c0-1.6055286 0-3.2070936-.012412-4.8097911-.006221-1.3484628-.024345-2.6972088-.061101-4.0459548-.079479-2.9384202-.249663-5.9045855-.774152-8.8101648-.524178-2.9534252-1.389119-5.7004618-2.754741-8.3795514-1.340798-2.6366228-3.090386-5.0439248-5.174771-7.13442444-2.091199-2.09049961-4.492257-3.84239716-7.125447-5.18180042-2.675884-1.36403401-5.419061-2.23035698-8.36242-2.76147524-2.908017-.52205868-5.87032-.69560638-8.801833-.77516088-1.353153-.03680457-2.700172-.05180951-4.04719-.06143532-1.596878-.0093427-3.199779-.0093427-4.796997-.0093427z"
        fill="#333"
      />
      <path
        d="m426.077344 10 4.725746.0090596c1.280186.0090595 2.5604.022932 3.847682.0580377 2.239223.0605858 4.858628.1820405 7.300254.6202965 2.122429.3827662 3.902447.9648426 5.610709 1.8356924 1.686464.8581098 3.231593 1.9826271 4.580707 3.3316515 1.354261 1.358084 2.478692 2.9078342 3.346946 4.6152773.864832 1.6972511 1.442442 3.4712253 1.822427 5.6121115.43653 2.418335.557254 5.0484372.618634 7.3045496.034464 1.2734338.051428 2.5468676.057676 3.8505942.011875 1.5766458.011875 3.1524422.011875 4.7322022v216.0641685c0 1.57976 0 3.152725-.012158 4.763061-.005965 1.273151-.022901 2.546585-.057676 3.822284-.061097 2.252715-.181793 4.881401-.623695 7.328331-.374641 2.111442-.95194 3.885699-1.821579 5.591444-.865737 1.703196-1.988444 3.251248-3.336795 4.600839-1.356777 1.358933-2.896816 2.480053-4.603326 3.34609-1.695511.866603-3.472476 1.448141-5.573982 1.826916-2.491076.444484-5.220235.566788-7.275657.622561-1.293191.02916-2.580163.046685-3.898488.052942-1.57114.011891-3.147878.011891-4.719272.011891h-384.1544332c-.0209218 0-.0412781 0-.0624826 0-1.5532997 0-3.1097094 0-4.6912818-.012174-1.2895158-.005974-2.5762044-.023215-3.8202013-.051527-2.1023547-.056905-4.8332106-.178926-7.303958-.620862-2.1195728-.381634-3.8965375-.963144-5.6143841-1.841072-1.6903938-.858393-3.2293027-1.97838-4.586673-3.339861-1.3469093-1.346477-2.4662238-2.889716-3.3316498-4.59546-.8662742-1.704046-1.4458636-3.483682-1.8269507-5.620322-.4405163-2.442116-.5615233-5.060328-.6219985-7.297755-.0345209-1.281077-.0489399-2.561872-.057139-3.835306l-.00622-3.759432v-.992588-216.0641685-.9925876l.0059373-3.7515053c.0084818-1.2810777.0229008-2.5618724.0574217-3.8418177.0604752-2.239692.1814822-4.8590359.6256456-7.3215363.3777227-2.117388.9570294-3.8970244 1.8278273-5.6095635.8586405-1.6927213 1.9799341-3.2379418 3.3339398-4.5934777 1.3486057-1.3510062 2.8925472-2.474108 4.5937129-3.340428 1.6994694-.8666031 3.4797986-1.4483964 5.5993996-1.8303133 2.442192-.4393884 5.06335-.56056 7.3081989-.6214289 1.2796204-.0348226 2.5592408-.0486951 3.8292485-.0574715l4.7416072-.0093427z"
        fill="#fff"
      />
    </g>
    <g transform="translate(23 94)">
      <g fill="#238ec2">
        <path
          d="m400.83873 2.69525159-13.453417 85.59022681c-.260388 1.6582649 1.02176 3.1566421 2.69982 3.1566421h13.529554c2.241475 0 4.149469-1.6308556 4.498177-3.8449211l13.26612-84.05073558c.261911-1.65978771-1.020237-3.1596876-2.69982-3.1596876h-15.140614c-1.346103 0-2.491204.97912247-2.69982 2.30847537"
        />
        <path
          d="m360.463252 61.2598709c-1.515127 8.9689446-8.635464 14.9898625-17.715569 14.9898625-4.552995 0-8.196915-1.4648768-10.540413-4.2377882-2.32218-2.7485476-3.196233-6.6635147-2.459227-11.0246449 1.413104-8.8882393 8.647647-15.1025454 17.590704-15.1025454 4.457063 0 8.075096 1.4770588 10.464277 4.2758568 2.404408 2.8185936 3.348507 6.7609701 2.660228 11.0992592zm21.875698-30.5507529h-15.696415c-1.34458 0-2.489681.9775997-2.698297 2.3069526l-.691324 4.3885396-1.096374-1.5897416c-3.40028-4.9352036-10.975917-6.5828094-18.540894-6.5828094-17.340975 0-32.1542 13.1412549-35.038271 31.5709894-1.4999 9.195833.630415 17.9820486 5.84428 24.1156494 4.789021 5.6341417 11.624605 7.9806856 19.768224 7.9806856 13.97724 0 21.731037-8.9796037 21.731037-8.9796037l-.701984 4.362653c-.261911 1.6582649 1.020237 3.1596876 2.69982 3.1596876h14.135605c2.239952 0 4.147946-1.6293329 4.498177-3.8433983l8.484713-53.7329621c.261911-1.658265-1.020237-3.1566421-2.698297-3.1566421z"
        />
        <path
          d="m288.146759 31.085235c-1.792266 11.7662198-10.777961 11.7662198-19.469767 11.7662198h-4.945863l3.468805-21.9685846c.210138-1.3278301 1.355239-2.3054299 2.699819-2.3054299h2.265839c5.917372 0 11.505831 0 14.386857 3.3698259 1.723743 2.0176318 2.246043 5.0067724 1.59431 9.1379688zm-3.782489-30.69845878h-32.780046c-2.241474 0-4.149469 1.63085562-4.498177 3.84492105l-13.25546 84.05225833c-.261911 1.658265 1.020236 3.1581649 2.698297 3.1581649h16.818675c1.568423 0 2.905389-1.1420558 3.149028-2.6906834l3.759647-23.8293741c.348708-2.2140654 2.256703-3.844921 4.498177-3.844921h10.372912c21.590945 0 34.051535-10.4490488 37.308678-31.1598493 1.466399-9.0572636.059387-16.1745551-4.181447-21.15696366-4.662633-5.47729942-12.925026-8.37355282-23.890284-8.37355282z"
        />
      </g>
      <path
        d="m232.141867 30.709118h-15.777119c-1.509037 0-2.920618.7491886-3.767262 1.9963134l-21.763014 32.0521756-9.223243-30.8004824c-.577118-1.9262674-2.351112-3.2480066-4.362653-3.2480066h-15.509117c-1.872971 0-3.190142 1.8425166-2.587137 3.6149871l17.372952 50.9905056-16.340533 23.0558213c-1.282148 1.809017.012182 4.312403 2.229293 4.312403h15.761892c1.493809 0 2.893208-.732439 3.742897-1.959768l52.469087-75.7243876c1.254739-1.8120618-.041114-4.2895614-2.246043-4.2895614"
        fill="#253667"
      />
      <path
        d="m126.667688 61.2598709c-1.51665 8.9689446-8.635464 14.9898625-17.717091 14.9898625-4.551473 0-8.195392-1.4648768-10.5388909-4.2377882-2.3221801-2.7485476-3.1962334-6.6635147-2.4592267-11.0246449 1.4131036-8.8882393 8.6461236-15.1025454 17.5891816-15.1025454 4.457063 0 8.076618 1.4770588 10.465799 4.2758568 2.404408 2.8185936 3.348507 6.7609701 2.660228 11.0992592zm21.874175-30.5507529h-15.694892c-1.346103 0-2.491204.9775997-2.699819 2.3069526l-.689802 4.3885396-1.097896-1.5897416c-3.398758-4.9352036-10.974395-6.5828094-18.539372-6.5828094-17.3409749 0-32.1541994 13.1412549-35.0382709 31.5709894-1.4998999 9.195833.6304148 17.9820486 5.8427572 24.1156494 4.7905433 5.6341417 11.6261277 7.9806856 19.7697467 7.9806856 13.97724 0 21.729514-8.9796037 21.729514-8.9796037l-.700461 4.362653c-.261911 1.6582649 1.020237 3.1596876 2.69982 3.1596876h14.134082c2.241475 0 4.149469-1.6293329 4.498177-3.8433983l8.486236-53.7329621c.261911-1.658265-1.020237-3.1566421-2.69982-3.1566421z"
        fill="#253667"
      />
      <path
        d="m54.3511949 31.085235c-1.7922661 11.7662198-10.7794836 11.7662198-19.4697665 11.7662198h-4.9458629l3.4688041-21.9685846c.2101382-1.3278301 1.3537167-2.3054299 2.698297-2.3054299h2.2673614c5.9158488 0 11.5058311 0 14.3868571 3.3698259 1.7237428 2.0176318 2.246043 5.0067724 1.5943098 9.1379688zm-3.7840114-30.69845878h-32.7800457c-2.2399521 0-4.1494693 1.63085562-4.498177 3.84492105l-13.2554605 84.05225833c-.26038871 1.658265 1.02023648 3.1581649 2.69981982 3.1581649h15.65073208c2.2414748 0 4.1494693-1.6308556 4.498177-3.8449211l3.5769186-22.6751364c.3502304-2.2140654 2.2582249-3.844921 4.4996997-3.844921h10.371389c21.5924676 0 34.0530573-10.4490488 37.3102003-31.1598493 1.4648769-9.0572636.0593869-16.1745551-4.1829696-21.15696366-4.6611102-5.47729942-12.9235029-8.37355282-23.8902837-8.37355282z"
        fill="#253667"
      />
    </g>
  </g>
</svg>
`,He=a`
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 449 300" aria-labelledby="venmoTitleID venmoDescID">
  <title id="venmoTitleID">Venmo icon</title>
  <desc id="venmoDescID">The Venmo logo</desc>
  <g fill="none">
    <rect width="448.934" height="299.289" fill="#3D95CE" rx="29.929" />
    <path
      fill="#FFF"
      d="M314.253648,95.768518 C314.253648,140.505629 276.917862,198.622312 246.615405,239.43135 L177.402732,239.43135 L149.644594,69.6528784 L210.247869,63.767475 L224.923984,184.575771 C238.636763,161.724586 255.559021,125.813905 255.559021,101.330492 C255.559021,87.9291341 253.314515,78.8010611 249.806862,71.285106 L304.995473,59.8578376 C311.376749,70.6382477 314.253648,81.742087 314.253648,95.768518 Z"
    />
  </g>
</svg>
`,J;(function(e){e.Loading=`loading`,e.Available=`available`,e.Unavailable=`unavailable`})(J||(J={}));var Y=class extends c{constructor(){super(...arguments),this.donationInfoValid=!0,this.applePayMode=J.Loading,this.googlePayMode=J.Loading,this.venmoMode=J.Loading,this.payPalMode=J.Loading,this.paymentModeSelected=void 0}render(){let e=this.paymentModeSelected?`payment-selected`:``;return a`
      <div
        class="payment-container ${this.donationInfoValid?`donation-info-valid`:`donation-info-invalid`} ${e}"
      >
        <div class="payment-provider-container">
          <button
            class="applepay provider-button ${this.applePayMode} ${this.paymentModeSelected===`apple`?`selected`:``}"
            @click=${e=>{this.paymentModeSelected=`apple`,this.applePaySelected(e)}}
            tabindex="0"
          >
            <div class="payment-image">${ze}</div>
          </button>

          <button
            class="googlepay provider-button ${this.googlePayMode} ${this.paymentModeSelected===`google`?`selected`:``}"
            @click=${()=>{this.paymentModeSelected=`google`,this.googlePaySelected()}}
            tabindex="0"
          >
            <div class="payment-image">${Be}</div>
          </button>

          <button
            class="venmo provider-button ${this.venmoMode} ${this.paymentModeSelected===`venmo`?`selected`:``}"
            @click=${()=>{this.paymentModeSelected=`venmo`,this.venmoSelected()}}
            tabindex="0"
          >
            <div class="payment-image">${He}</div>
          </button>

          <div
            class="paypal-container provider-button ${this.payPalMode}
              ${this.paymentModeSelected===`paypal`?`selected`:``}"
            tabindex="0"
          >
            <div class="payment-image">
              <div
                class="paypal-local-button"
                @click=${()=>{this.paymentModeSelected=`paypal`,this.localPaypalButtonClicked()}}
              >
                ${Ve}
              </div>
              <slot name="paypal-button"></slot>
            </div>
          </div>
        </div>

        <div class="credit-card-container">
          <button
            @click=${()=>{this.paymentModeSelected=`cc`,this.creditCardSelected()}}
            class="button-style credit-card-button ${this.paymentModeSelected===`cc`?`selected`:``}"
            tabindex="0"
          >
            <div class="cc-title">Credit Card</div>
            <div class="cc-background"></div>
          </button>
        </div>
      </div>

      ${this.paymentModeSelected?a`
            <button
              id="change-payment-method"
              @click=${()=>{this.paymentModeSelected=void 0,this.dispatchEvent(new Event(`resetPaymentMethod`)),this.setButtonVisibility()}}
            >
              Change payment method
            </button>

            <slot name="credit-card-fields"></slot>
          `:i}
    `}firstUpdated(){this.dispatchEvent(new Event(`firstUpdated`))}updated(e){e.has(`paymentProviders`)&&this.setButtonVisibility()}showPaypalButton(){this.payPalMode=J.Available}setButtonVisibility(){return f(this,void 0,void 0,function*(){var e,t,n;(e=this.paymentProviders)==null||e.venmoHandler.get().then(e=>{if(!e){this.venmoMode=J.Unavailable;return}e.isBrowserSupported().then(e=>{this.venmoMode=e?J.Available:J.Unavailable}).catch(e=>{console.error(`error loading venmo`,e),this.venmoMode=J.Unavailable})}).catch(e=>{console.error(`venmo unavailable`,e),this.venmoMode=J.Unavailable}),(t=this.paymentProviders)==null||t.applePayHandler.get().then(e=>{if(!e){console.error(`applePayHandler unavailable`),this.applePayMode=J.Unavailable;return}e.isAvailable().then(e=>{this.applePayMode=e?J.Available:J.Unavailable}).catch(e=>{console.error(`error loading applepay`,e),this.applePayMode=J.Unavailable})}).catch(e=>{console.error(`apple pay unavailable`,e),this.applePayMode=J.Unavailable}),(n=this.paymentProviders)==null||n.googlePayHandler.get().then(e=>{if(!e){console.error(`google pay handler unavailable`),this.googlePayMode=J.Unavailable;return}e.isBrowserSupported().then(e=>{this.googlePayMode=e?J.Available:J.Unavailable}).catch(e=>{console.error(`error loading googlepay`,e),this.googlePayMode=J.Unavailable})}).catch(e=>{console.error(`google pay unavailable`,e),this.googlePayMode=J.Unavailable})})}googlePaySelected(){this.dispatchEvent(new Event(`googlePaySelected`))}applePaySelected(e){let t=new CustomEvent(`applePaySelected`,{detail:{originalEvent:e}});this.dispatchEvent(t)}venmoSelected(){this.dispatchEvent(new Event(`venmoSelected`))}creditCardSelected(){this.dispatchEvent(new Event(`creditCardSelected`))}localPaypalButtonClicked(){this.dispatchEvent(new Event(`paypalBlockerSelected`))}static get styles(){let e=n`var(--paymentButtonWidth, 5rem)`,t=n`var(--paymentButtonHeight, 3.2rem)`;return n`
      button {
        color: inherit;
        font-family: inherit;
      }

      .payment-container {
        width: 100%;
      }

      .payment-provider-container {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr 1fr;
        grid-gap: 1rem;
        margin-bottom: 1rem;
        max-width: 23rem;
      }

      .provider-button {
        border: 0;
        padding: 0;
        background: none;
        cursor: pointer;
        width: ${e};
        height: ${t};
      }

      .provider-button.unavailable {
        display: none;
      }

      .provider-button.loading {
        border: 1px solid #ddd;
        border-radius: 2px;
        /* account for the borders that don't exist once the provider loads, otherwise the layout shifts */
        margin-bottom: -2px;
      }

      .provider-button.loading .payment-image {
        display: none;
      }

      .paypal-local-button {
        position: absolute;
        width: ${e};
        height: ${t};
      }

      .donation-info-valid .paypal-local-button {
        z-index: 0;
      }

      .donation-info-invalid .paypal-local-button {
        z-index: 250;
      }

      .credit-card-button {
        color: var(--ccButtonFontColor, #333);
        background-color: var(--ccButtonColor, white);
        border: 1px solid #333;
        border-radius: 4px;
        cursor: pointer;
        margin: 0;
        padding: 0.7rem 1rem;
        width: 100%;
      }

      .credit-card-button .cc-background {
        height: 2.4rem;
        width: 100%;
        background-repeat: no-repeat;
        background-image: url(https://archive.org/images/cc_logos.png);
        background-position: 50% 50%;
        background-size: contain;
      }

      .credit-card-button .cc-title {
        font-size: ${n`var(--creditCardFontSize, 1.8rem)`};
        font-weight: 700;
        margin-bottom: 0.5rem;
      }

      button#change-payment-method {
        margin-top: 10px;
        margin-bottom: 10px;
        background: white;
        border: 1px solid;
        border-radius: 3px;
        padding: 5px;
      }

      button#change-payment-method:hover {
        cursor: pointer;
      }

      .payment-selected button:not(.selected),
      .payment-selected .paypal-container:not(.selected) {
        display: none;
      }
    `}};p([r({type:Boolean})],Y.prototype,`donationInfoValid`,void 0),p([r({type:Object})],Y.prototype,`paymentProviders`,void 0),p([r({type:String})],Y.prototype,`applePayMode`,void 0),p([r({type:String})],Y.prototype,`googlePayMode`,void 0),p([r({type:String})],Y.prototype,`venmoMode`,void 0),p([r({type:String})],Y.prototype,`payPalMode`,void 0),p([r({type:String})],Y.prototype,`paymentModeSelected`,void 0),Y=p([u(`payment-selector`)],Y);var X;(function(e){e.Summary=`summary`,e.Edit=`edit`})(X||(X={}));var Z=class extends c{constructor(){super(...arguments),this.mode=X.Edit,this.amountOptions=S,this.amountSelectionLayout=G.MultiLine,this.frequencySelectionMode=W.Button}render(){return a` ${this.currentTemplate} `}get currentTemplate(){switch(this.mode){case X.Summary:return this.donationSummaryTemplate;case X.Edit:return this.editDonationTemplate}}get editDonationTemplate(){return a`
      <donation-form-edit-donation
        .donationInfo=${this.donationInfo}
        .amountOptions=${this.amountOptions}
        .amountSelectionLayout=${this.amountSelectionLayout}
        .frequencySelectionMode=${this.frequencySelectionMode}
        @donationInfoChanged=${this.donationInfoChanged}
        @showSummaryClicked=${this.showSummaryClicked}
        @editDonationError=${this.editDonationError}
      >
      </donation-form-edit-donation>
    `}get donationSummaryTemplate(){return a`
      <donation-summary .donationInfo=${this.donationInfo} @editClicked=${this.summaryEditClicked}>
      </donation-summary>
    `}donationInfoChanged(e){this.donationInfo=e.detail.donationInfo;let t=new CustomEvent(`donationInfoChanged`,{detail:{donationInfo:this.donationInfo}});this.dispatchEvent(t)}editDonationError(e){let t=new CustomEvent(`editDonationError`,{detail:e.detail});this.dispatchEvent(t)}summaryEditClicked(){this.mode=X.Edit}showSummaryClicked(){this.mode=X.Summary}static get styles(){return n``}};p([r({type:Object})],Z.prototype,`donationInfo`,void 0),p([r({type:String})],Z.prototype,`mode`,void 0),p([r({type:Array})],Z.prototype,`amountOptions`,void 0),p([r({type:String})],Z.prototype,`amountSelectionLayout`,void 0),p([r({type:String})],Z.prototype,`frequencySelectionMode`,void 0),p([d(`edit-donation`)],Z.prototype,`editDonation`,void 0),Z=p([u(`donation-form-header`)],Z);var Q=class extends c{render(){return a`
      <div class="top-line"></div>
      <div class="total-line">Total: ${this.totalAmount}</div>
    `}get totalAmount(){if(this.donationInfo)return`${(0,D.default)(this.donationInfo.total,{symbol:`$`}).format()}${this.donationInfo.donationType===x.Monthly?`/month`:``}`}static get styles(){let e=n`var(--totalAmountLineColor, #333)`,t=n`var(--totalAmountLineThickness, 2px)`,r=n`var(--totalAmountVerticalSpacing, 0.5rem)`;return n`
      .top-line {
        width: 100%;
        height: ${t};
        background-color: ${e};
      }

      .total-line {
        font-size: ${n`var(--totalAmountFontSize, 2.6rem)`};
        font-weight: bold;
        text-align: center;
        margin-top: ${r};
      }
    `}};p([r({type:Object})],Q.prototype,`donationInfo`,void 0),Q=p([u(`donation-form-total-amount`)],Q);var $=class extends c{constructor(){super(...arguments),this.amountOptions=S,this.amountSelectionLayout=G.MultiLine,this.frequencySelectionMode=W.Button,this.creditCardVisible=!1,this.contactFormVisible=!1,this.donationInfoValid=!0,this.paypalButtonNeedsRender=!0,this.flowHandlersConfigured=!1,this.flowHandlerListenersBound=!1}render(){return a`
      <donation-form-header
        .amountOptions=${this.amountOptions}
        .amountSelectionLayout=${this.amountSelectionLayout}
        .frequencySelectionMode=${this.frequencySelectionMode}
        @donationInfoChanged=${this.donationInfoChanged}
        @editDonationError=${this.editDonationError}
      >
      </donation-form-header>

      <donation-form-section
        .badgeMode=${z.HideBadgeLeaveSpacing}
        id="total-amount-section"
      >
        <donation-form-total-amount .donationInfo=${this.donationInfo}>
        </donation-form-total-amount>
      </donation-form-section>

      <donation-form-section
        .sectionBadge=${this.paymentSelectorNumberingStart}
        headline="Choose a payment method"
      >
        <payment-selector
          .paymentProviders=${this.braintreeManager?.paymentProviders}
          @firstUpdated=${this.paymentSelectorFirstUpdated}
          @creditCardSelected=${this.creditCardSelected}
          @venmoSelected=${this.venmoSelected}
          @applePaySelected=${this.applePaySelected}
          @googlePaySelected=${this.googlePaySelected}
          @paypalBlockerSelected=${this.paypalBlockerSelected}
          @resetPaymentMethod=${()=>f(this,void 0,void 0,function*(){this.selectedPaymentProvider=void 0,this.contactFormVisible=!1,this.requestUpdate()})}
          tabindex="0"
        >
          <slot name="paypal-button" slot="paypal-button"></slot>
          <div
            class="credit-card-fields ${this.creditCardVisible?``:`hidden`}"
            slot="credit-card-fields"
          >
            <slot name="braintree-hosted-fields"></slot>
          </div>
        </payment-selector>
      </donation-form-section>

      <div class="contact-form-section ${this.contactFormVisible?``:`hidden`}">
        ${this.contactFormSectionTemplate}
      </div>
      <slot name="recaptcha"></slot>
    `}showConfirmationModalDev(e){return f(this,void 0,void 0,function*(){var t;(t=this.paymentFlowHandlers)==null||t.showConfirmationStepModal(e)})}showUpsellModalDev(e){return f(this,void 0,void 0,function*(){var t;if((t=this.paymentFlowHandlers)==null||t.showUpsellModal(e),e.ctaMode===k.PayPalUpsellSlot){let t=yield this.braintreeManager?.paymentProviders.paypalHandler.get(),n=new b({amount:e.oneTimeAmount,donationType:x.OneTime,coverFees:!1});t?.renderPayPalButton({selector:`#paypal-upsell-button`,style:{color:`blue`,label:`paypal`,shape:`rect`,size:`responsive`,tagline:!1},donationInfo:n})}})}get contactFormSectionTemplate(){let e=this.selectedPaymentProvider===y.Venmo?`Help us stay in touch`:`Enter contact information`;return a`
      <donation-form-section
        .sectionBadge=${this.paymentSelectorNumberingStart+1}
        headline=${e}
        id="contactFormSection"
      >
        <slot name="contact-form"></slot>
      </donation-form-section>

      <donation-form-section .sectionBadge=${this.paymentSelectorNumberingStart+2}>
        <slot name="recaptcha"></slot>
        <button id="donate-button" @click=${this.donateClicked}>Donate</button>

        <div class="secure-process-note">${L} Your payment will be securely processed</div>
      </donation-form-section>
    `}get paymentSelectorNumberingStart(){return this.frequencySelectionMode===W.Button?3:2}editDonationError(){this.donationInfoValid=!1}paymentSelectorFirstUpdated(){this.paymentFlowHandlers?.paypalHandler&&this.renderPayPalButtonIfNeeded()}applePaySelected(e){var t;if(this.selectedPaymentProvider=y.ApplePay,this.contactFormVisible=!1,this.creditCardVisible=!1,!this.donationInfoValid){this.showInvalidDonationInfoAlert();return}let n=e.detail.originalEvent;this.donationInfo&&((t=this.paymentFlowHandlers?.applePayHandler)==null||t.paymentInitiated(this.donationInfo,n)),this.emitPaymentFlowStartedEvent()}googlePaySelected(){var e;this.selectedPaymentProvider=y.GooglePay,this.contactFormVisible=!1,this.creditCardVisible=!1,this.donationInfoValid?(this.donationInfo&&((e=this.paymentFlowHandlers?.googlePayHandler)==null||e.paymentInitiated(this.donationInfo)),this.emitPaymentFlowStartedEvent()):this.showInvalidDonationInfoAlert()}creditCardSelected(){return f(this,void 0,void 0,function*(){if(!this.donationInfoValid){this.showInvalidDonationInfoAlert();return}this.selectedPaymentProvider=y.CreditCard,this.contactFormVisible=!0,this.creditCardVisible=!0,this.focusCreditCardNumberField()})}venmoSelected(){return f(this,void 0,void 0,function*(){if(!this.donationInfoValid){this.showInvalidDonationInfoAlert();return}this.selectedPaymentProvider=y.Venmo,this.contactFormVisible=!0,this.creditCardVisible=!1,this.focusContactForm()})}paypalBlockerSelected(){this.contactFormVisible=!1,this.creditCardVisible=!1,this.showInvalidDonationInfoAlert()}focusContactForm(){return f(this,void 0,void 0,function*(){var e;yield this.updateComplete,this.contactFormSection&&((e=this.contactForm)==null||e.focus())})}focusCreditCardNumberField(){return f(this,void 0,void 0,function*(){yield this.updateComplete,(yield this.braintreeManager?.paymentProviders.creditCardHandler.get())?.focusField(O.Number)})}donateClicked(){return f(this,void 0,void 0,function*(){if(!this.contactForm){alert(`Please enter contact info.`);return}if(!this.donationInfoValid||!this.donationInfo){this.showInvalidDonationInfoAlert();return}let e=this.contactForm.donorContactInfo;switch(this.selectedPaymentProvider){case y.CreditCard:this.handleCreditCardDonationFlow(e,this.donationInfo);break;case y.Venmo:this.handleVenmoDonationFlow(e,this.donationInfo);break}})}handleCreditCardDonationFlow(e,t){return f(this,void 0,void 0,function*(){let n=this.paymentFlowHandlers?.creditCardHandler;(yield this.braintreeManager?.paymentProviders.creditCardHandler.get())?.hideErrorMessage();let r=this.contactForm?.reportValidity(),i=yield n?.tokenizeFields();!r||i===void 0||(this.emitPaymentFlowStartedEvent(),n?.paymentInitiated(i,t,e))})}handleVenmoDonationFlow(e,t){return f(this,void 0,void 0,function*(){var n;this.contactForm?.reportValidity()&&((n=this.paymentFlowHandlers?.venmoHandler)==null||n.paymentInitiated(e,t))})}emitPaymentFlowStartedEvent(){if(!this.selectedPaymentProvider)return;let e=new CustomEvent(`paymentFlowStarted`,{detail:{paymentProvider:this.selectedPaymentProvider}});this.dispatchEvent(e)}emitPaymentFlowConfirmedEvent(){if(!this.selectedPaymentProvider)return;let e=new CustomEvent(`paymentFlowConfirmed`,{detail:{paymentProvider:this.selectedPaymentProvider}});this.dispatchEvent(e)}emitPaymentFlowCancelledEvent(){if(!this.selectedPaymentProvider)return;let e=new CustomEvent(`paymentFlowCancelled`,{detail:{paymentProvider:this.selectedPaymentProvider}});this.dispatchEvent(e)}emitPaymentFlowErrorEvent(e){if(!this.selectedPaymentProvider)return;let t=new CustomEvent(`paymentFlowError`,{detail:{paymentProvider:this.selectedPaymentProvider,error:e}});this.dispatchEvent(t)}showInvalidDonationInfoAlert(){alert(`Please enter a valid donation amount.`)}renderPayPalButtonIfNeeded(){return f(this,void 0,void 0,function*(){this.paypalButtonNeedsRender&&(this.paypalButtonNeedsRender=!1,this.donationInfo&&(yield(this.paymentFlowHandlers?.paypalHandler)?.renderPayPalButton(this.donationInfo)),this.paymentSelector.showPaypalButton())})}updated(e){var t;if(e.has(`donationInfo`)&&this.donationInfo&&((t=this.paymentFlowHandlers?.paypalHandler)==null||t.updateDonationInfo(this.donationInfo),this.donationFormHeader.donationInfo=this.donationInfo),(e.has(`paymentFlowHandlers`)||e.has(`donationInfo`))&&this.donationInfo&&this.paymentFlowHandlers&&this.setupFlowHandlers(),e.has(`donationInfoValid`)&&(this.paymentSelector.donationInfoValid=this.donationInfoValid),e.has(`selectedPaymentProvider`)){let t=new CustomEvent(`paymentProviderSelected`,{detail:{paymentProvider:this.selectedPaymentProvider,previousPaymentProvider:e.get(`selectedPaymentProvider`)}});this.dispatchEvent(t)}}setupFlowHandlers(){var e;this.flowHandlersConfigured||(this.flowHandlersConfigured=!0,this.bindFlowListenerEvents(),this.renderPayPalButtonIfNeeded(),this.donationInfo&&((e=this.paymentFlowHandlers?.paypalHandler)==null||e.updateDonationInfo(this.donationInfo)))}bindFlowListenerEvents(){var e,t,n,r,i;this.flowHandlerListenersBound||(this.flowHandlerListenersBound=!0,(e=this.paymentFlowHandlers?.paypalHandler)==null||e.on(`payPalPaymentStarted`,()=>{this.selectedPaymentProvider=y.PayPal,this.emitPaymentFlowStartedEvent()}),(t=this.paymentFlowHandlers?.paypalHandler)==null||t.on(`payPalPaymentConfirmed`,()=>{this.selectedPaymentProvider=y.PayPal,this.emitPaymentFlowConfirmedEvent()}),(n=this.paymentFlowHandlers?.paypalHandler)==null||n.on(`payPalPaymentCancelled`,()=>{this.selectedPaymentProvider=y.PayPal,this.emitPaymentFlowCancelledEvent()}),(r=this.paymentFlowHandlers?.paypalHandler)==null||r.on(`payPalPaymentError`,(e,t)=>{this.selectedPaymentProvider=y.PayPal,this.emitPaymentFlowErrorEvent(t)}),(i=this.paymentFlowHandlers?.googlePayHandler)==null||i.on(`paymentCancelled`,()=>{this.selectedPaymentProvider=y.GooglePay,this.emitPaymentFlowCancelledEvent()}))}donationInfoChanged(e){let t=e.detail.donationInfo;this.donationInfo=new b({amount:t.amount,donationType:t.donationType,coverFees:t.coverFees}),this.donationInfoValid=!0;let n=new CustomEvent(`donationInfoChanged`,{detail:{donationInfo:t}});this.dispatchEvent(n)}static get styles(){let e=n`var(--donateButtonFontSize, 2.6rem)`,t=n`var(--donateButtonHeight, 4rem)`,r=n`var(--donateButtonColor, rgba(49, 164, 129, 1))`;return n`
      h1 {
        margin: 0;
        padding: 0;
      }

      .hidden {
        display: none;
      }

      .secure-process-note {
        margin-top: 0.5em;
        font-size: 0.75em;
        text-align: center;
      }

      .secure-process-note svg {
        width: 1.2rem;
        height: 1.5rem;
        vertical-align: bottom;
      }

      #donate-button {
        width: 100%;
        appearance: none;
        -webkit-appearance: none;
        font-size: ${e};
        font-weight: bold;
        text-align: center;
        color: ${n`var(--donateButtonTextColor, #fff)`};
        cursor: pointer;
        border: none;
        border-radius: 5px;
        background-color: ${r};
        padding-top: 0.5rem;
        padding-bottom: 0.5rem;
        height: ${t};
      }

      #donate-button:hover {
        background-color: ${n`var(--donateButtonHoverColor, rgba(39, 131, 103, 1))`};
      }

      #total-amount-section {
        display: block;
        margin-top: ${n`var(--donateTotalAmountTopMargin, 1.5rem)`};
        margin-bottom: ${n`var(--donateTotalAmountBottomMargin, 1.2rem)`};
      }
    `}};p([r({type:Object})],$.prototype,`braintreeManager`,void 0),p([r({type:Object})],$.prototype,`paymentFlowHandlers`,void 0),p([r({type:Object})],$.prototype,`donationRequest`,void 0),p([r({type:Object})],$.prototype,`donationInfo`,void 0),p([r({type:Object})],$.prototype,`contactForm`,void 0),p([r({type:Array})],$.prototype,`amountOptions`,void 0),p([r({type:String})],$.prototype,`amountSelectionLayout`,void 0),p([r({type:String})],$.prototype,`frequencySelectionMode`,void 0),p([r({type:Boolean})],$.prototype,`creditCardVisible`,void 0),p([r({type:Boolean})],$.prototype,`contactFormVisible`,void 0),p([r({type:Boolean})],$.prototype,`donationInfoValid`,void 0),p([r({type:String})],$.prototype,`selectedPaymentProvider`,void 0),p([d(`#contactFormSection`)],$.prototype,`contactFormSection`,void 0),p([d(`donation-form-header`)],$.prototype,`donationFormHeader`,void 0),p([d(`payment-selector`)],$.prototype,`paymentSelector`,void 0),$=p([u(`donation-form`)],$);export{E as a,b as c,me as i,y as l,O as n,w as o,be as r,x as s,L as t};