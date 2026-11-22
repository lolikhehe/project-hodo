import{n as e}from"./rolldown-runtime-QTnfLwEv.js";import{A as t,C as n,O as r,T as i,w as a,x as o}from"./vendor-lit-MQ4DzmMc.js";import{c as s,s as c}from"./vendor-shared-62wh2Xkx.js";import{a as l}from"./dist-P4PnKJsG.js";var u=t`var(--white, #fff)`,d=t`var(--primaryDisableCTAFill, #767676)`,f=t`var(--secondaryCTABorder, #999)`,p=t`var(--primaryCTAFill, #194880)`,m=t`var(--primaryCTAFillRGB, 25, 72, 128)`,h=t`var(--primaryCTABorder, #c5d1df)`,g=t`var(--primaryErrorCTAFill, #d9534f)`,_=t`var(--primaryErrorCTAFillRGB, 229, 28, 38)`,v=t`var(--primaryErrorCTABorder, #d43f3a)`,y=t`var(--secondaryCTAFill, #333)`,b=t`var(--secondaryCTAFillRGB, 51, 51, 51)`,x=t`var(--primaryCTABorder, #979797)`,S=t`
  .ia-button {
    min-height: 3rem;
    cursor: pointer;
    color: ${u};
    line-height: normal;
    border-radius: 0.4rem;
    font-size: 1.4rem;
    font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
    border: 1px solid transparent;
    white-space: nowrap;
    appearance: auto;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    transition: all 0.1s ease 0s;
    vertical-align: middle;
    padding: 0 3rem;
    outline-color: ${u};
    outline-offset: -4px;
    user-select: none;
    text-decoration: none;
    width: fit-content;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    -o-user-select: none;
  }
  .ia-button:focus-visible {
    outline-style: double;
  }
  .ia-button:disabled {
    cursor: not-allowed;
    background-color: ${d};
    border: 1px solid ${f};
  }
  .ia-button.transparent {
    background-color: transparent;
  }
  .ia-button.warning {
    background-color: ${t`#ee8950`}
    border-color: ${t`#ec7939`};
  }

  .ia-button.primary {
    background-color: ${p};
    border-color: ${h};
  }
  .ia-button.primary:hover {
    background-color: rgba(${m}, 0.9);
  }
  .ia-button.primary:focus-visible {
    background-color: rgba(${m}, 0.8);
  }
  .ia-button.primary:active {
    background-color: rgba(${m}, 0.7);
  }

  .ia-button.danger {
    background-color: ${g};
    border-color: ${v};
  }
  .ia-button.danger:hover {
    background-color: rgba(${_}, 0.9);
  }
  .ia-button.danger:focus-visible {
    background-color: rgba(${_}, 0.8);
  }
  .ia-button.danger:active {
    background-color: rgba(${_}, 0.7);
  }

  .ia-button.dark {
    background-color: ${y};
    border-color: ${x};
  }
  .ia-button.dark:hover {
    background-color: rgba(${b}, 0.9);
  }
  .ia-button.dark:focus-visible {
    background-color: rgba(${b}, 0.8);
  }
  .ia-button.dark:active {
    background-color: rgba(${b}, 0.7);
  }
`,C;(function(e){e.NETWORK=`failed to connect to user lists backend service`,e.BAD_RESPONSE=`malformed response from backend`,e.USER_NOT_FOUND=`user with the given id was not found`,e.LIST_NOT_FOUND=`user list with the given id was not found`,e.LIST_MEMBER_NOT_FOUND=`user list member with the given id was not found`,e.ITEM_NOT_TOUND=`item with the given id was not found`})(C||(C={}));var w=class extends Error{constructor(e,t,n){super(t,n),this.reason=e}},T=`application/json; charset=utf-8`,E=class e{constructor({fetchHandler:t,searchService:n,userService:r,baseUrl:i=e.DEFAULT_BASE_URL}){this.fetchHandler=t,this.searchService=n,this.userService=r,this.baseUrl=i}async fetchEndpoint(t,n,r,i){try{let a=await this.fetchHandler.fetchApiResponse(t,{method:n,body:r,headers:i,includeCredentials:!0});return a.success?{success:a.value}:{error:e.getErrorResult(C.LIST_NOT_FOUND,a.error)}}catch(t){return{error:e.getErrorResult(C.NETWORK,t)}}}async fetchListsForUser(e){let t=(await this.userService.getLoggedInUser())?.success?.itemname===e?`me`:e;return this.fetchEndpoint(`${this.baseUrl}/services/users/${t}/lists`)}async fetchList(e,t){return this.fetchEndpoint(`${this.baseUrl}/services/users/${e}/lists/${t}`)}async fetchOwnListsContainingItem(e){return this.fetchEndpoint(`${this.baseUrl}/services/users/me/lists?item=${e}`)}async fetchListMembers(t,n){let r=await this.fetchList(t,n);if(!r.success)return r;let{members:i}=r.success;if(!i)throw e.getErrorResult(C.BAD_RESPONSE);return{success:i}}async fetchListMemberSearchResults(e,t){let n=await this.fetchListMembers(e,t);if(!n.success)return n;let r=n.success;if(r.length===0)return{success:[]};let i=`identifier:(${r.map(e=>e.identifier).join(` OR `)})`,a=await this.searchService.search({query:i,rows:r.length,aggregations:{omit:!0}},l.METADATA);return a.success?{success:a.success.response.results}:{error:a.error}}async createList(e){return this.fetchEndpoint(`${this.baseUrl}/services/users/me/lists`,`POST`,JSON.stringify(e),{"Content-Type":T})}async updateList(e,t){return this.fetchEndpoint(`${this.baseUrl}/services/users/me/lists/${e}`,`PATCH`,JSON.stringify(t),{"Content-Type":T})}async deleteList(e){return this.fetchEndpoint(`${this.baseUrl}/services/users/me/lists/${e}`,`DELETE`)}async addMemberToList(e,t){return this.fetchEndpoint(`${this.baseUrl}/services/users/me/lists/${e}/members`,`POST`,JSON.stringify(t),{"Content-Type":T})}async removeMemberFromList(e,t){return this.fetchEndpoint(`${this.baseUrl}/services/users/me/lists/${e}/members/${t}`,`DELETE`)}async removeBulkMembersFromList(e,t){return this.fetchEndpoint(`${this.baseUrl}/services/users/me/lists/${e}/members`,`PATCH`,JSON.stringify({remove:t}),{"Content-Type":T})}static getErrorResult(t,n){return new w(t,e.getErrorMessage(n),{cause:n})}static getErrorMessage(e){return e instanceof Error?e.message:typeof e==`string`?e:`Unknown error`}};E.DEFAULT_BASE_URL=`https://archive.org`;var D=class extends i{async saveListDetails(e){e.preventDefault();let t=e.target?.querySelector(`button#save-list-settings`);t?.setAttribute(`disabled`,`true`),this.dispatchEvent(new Event(`userListSaving`));try{let e={list_name:this.listName.value,description:this.listDescription.value,is_private:this.listPrivate.checked},t;if(t=this.userList?.id?await this.userListsService?.updateList(this.userList.id,e):await this.userListsService?.createList(e),t.success)this.dispatchEvent(new CustomEvent(`userListSaved`,{detail:t.success}));else throw t.error}catch(e){this.dispatchEvent(new CustomEvent(`userListError`,{detail:{error:e}})),console.log(`error`,e),t?.removeAttribute(`disabled`)}}emitCloseModalEvent(e){e.preventDefault(),this.dispatchEvent(new Event(`listModalClosed`))}render(){return r`
      <section class="new-list">
        <form id="user-list-form" @submit=${this.saveListDetails}>
          <div class="field">
            <input type="hidden" id="id" .value=${this.userList?.id??``} />
            <label for="name">List name*</label>
            <input
              type="text"
              id="name"
              .value=${this.userList?.list_name??``}
              required
            />
          </div>
          <div class="field">
            <label for="description">Description</label>
            <textarea
              id="description"
              .value=${this.userList?.description??``}
            ></textarea>
          </div>
          <div class="field">
            <label for="private">Private list</label>
            <input
              type="checkbox"
              id="private"
              .checked="${this.userList?.is_private??!1}"
            />
          </div>
          <div class="footer field">
            <button
              type="button"
              class="ia-button dark"
              id="cancel"
              @click=${this.emitCloseModalEvent}
            >
              Cancel
            </button>
            <button
              type="submit"
              id="save-list-settings"
              class="ia-button primary"
            >
              Save
            </button>
          </div>
        </form>
      </section>
    `}};D.styles=[S,t`
      :host {
        display: block;
        padding: 15px 0 0 0;
        font-size: 14px;
        position: relative;
      }

      .field {
        margin-bottom: 10px;
        display: flex;
        align-items: center;
      }

      label {
        display: inline-block;
        width: 90px;
        text-align: left;
        margin-right: 10px;
        font-weight: 700;
      }

      input[type='text'],
      textarea {
        line-height: 20px;
        padding: 5px;
        border-radius: 5px;
        width: 270px;
        font-family: inherit;
        font-size: inherit;
        resize: none;
        border-style: solid;
        outline: none;
        border-radius: 4px;
        border-width: 1px;
        border-color: #999;
      }

      input[type='checkbox'] {
        cursor: pointer;
        margin: 0;
      }

      label[for='private'] {
        cursor: pointer;
      }

      .footer {
        display: block;
        text-align: center;
        margin: 10px 0 0 0;
      }

      .ia-button {
        display: initial;
        padding: 0 15px;
        height: 3.5rem;
      }
    `],c([n({type:Object}),s(`design:type`,Object)],D.prototype,`userList`,void 0),c([n({type:Object}),s(`design:type`,E)],D.prototype,`userListsService`,void 0),c([o(`#id`),s(`design:type`,HTMLInputElement)],D.prototype,`listId`,void 0),c([o(`#name`),s(`design:type`,HTMLInputElement)],D.prototype,`listName`,void 0),c([o(`#description`),s(`design:type`,HTMLInputElement)],D.prototype,`listDescription`,void 0),c([o(`#private`),s(`design:type`,HTMLInputElement)],D.prototype,`listPrivate`,void 0),D=c([a(`iaux-userlist-settings`)],D);var O=e({IAUserListSettings:()=>D,UserListsError:()=>w,UserListsErrorReason:()=>C,UserListsService:()=>E});export{w as n,O as t};