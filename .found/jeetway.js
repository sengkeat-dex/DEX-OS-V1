const __vite__mapDeps = (i, m=__vite__mapDeps, d=(m.f || (m.f = ["./B-wr1a5f.js", "./DNO-EXHB.js", "./entry.BhmX-gm6.css"]))) => i.map(i => d[i]);
import {aM as e, E as a, a6 as l, s as n, a7 as t, h as s, p as i, q as o, a8 as d, c as r, o as p, i as u, a as c, j as m, F as y, r as v, n as g, H as b, G as f, e as h, t as x, w as k, aN as T, ab as w, aO as I, k as P, aP as j, aQ as _, aR as L, d as $, _ as C} from "./DNO-EXHB.js";
import {u as M, _ as z} from "./C8_z2_qL.js";
import O from "./D3EU2Nc7.js";
import R from "./qbIEXZRc.js";
import B from "./DnPVw45V.js";
const D = Object.freeze({
    bKash: "/images/img/payment/bKash.png",
    bKash_light: "/images/img/payment/bKash_light.svg",
    Nagad: "/images/img/payment/Nagad.png",
    Rocket: "/images/img/payment/Rocket.png",
    Cryptoexchange: "/images/img/payment/Cryptoexchange.png",
    "Online Banking": "/images/img/payment/Online_Banking.svg",
    rupeeo: "/images/img/payment/rupeeo.png",
    UPay: "/images/img/payment/UPay.png",
    UPI: "/images/img/payment/UPI.svg"
})
  , U = $(( () => C(( () => import("./B-wr1a5f.js")), __vite__mapDeps([0, 1, 2]), import.meta.url).then((e => e.default || e))))
  , E = {
    class: "space-y-2 lg:font-bold w-full"
}
  , N = {
    key: 0,
    class: "space-y-[10px]"
}
  , S = {
    class: "flex gap-[10px] w-full"
}
  , K = {
    key: 1,
    class: "space-y-2"
}
  , V = {
    class: "flex flex-col space-y-2 lg:flex-row bg-gray-50 dark:bg-gray-800 rounded-md px-[10px] py-3 items-center"
}
  , q = {
    class: "w-full lg:w-[40%] font-bold"
}
  , A = {
    class: "space-y-2 w-full font-bold bg-gray-50 dark:bg-gray-800 rounded-md px-[10px] py-3 flex flex-col gap-1"
}
  , H = {
    class: "grid grid-cols-3 lg:grid-cols-3 xl:grid-cols-4 gap-[10px] w-full"
}
  , W = ["onClick"]
  , F = {
    class: "absolute -top-1 -right-1 chip bg-primary shadow rounded-r-md"
}
  , G = ["innerHTML"]
  , Q = {
    class: "text-center justify-items-end text-xs sm:text-sm lg:text-[13px]"
}
  , X = {
    key: 0,
    class: "hidden"
}
  , J = {
    class: "grid grid-cols-2 md:flex w-full gap-2"
}
  , Y = ["onClick"]
  , Z = {
    class: "text-center justify-items-end"
}
  , ee = {
    key: 0
}
  , ae = {
    key: 1
}
  , le = {
    key: 3
}
  , ne = {
    __name: "index",
    setup($) {
        const C = e()
          , ne = a()
          , te = M()
          , se = l()
          , {depositPaymentInfo: ie, selectedBankTypeId: oe, bonusTemplateOptions: de, selectedBonusTemplateId: re, paymentTypeList: pe, selectedPaymentMethodTypeId: ue, selectedDepositTypeId: ce} = n(C)
          , {verifyOtpCode: me} = n(se)
          , ye = t("")
          , ve = s();
        console.log("colorMode", ve.value);
        const ge = e => {
            const a = "light" === ve.value ? "_light" : ""
              , l = `${e}${a}`;
            if (D[l])
                return D[l];
            if (D[e])
                return D[e];
            const n = D[l] ? a : "";
            return `/images/img/payment/${e.replace(/\s+/g, "_")}${n}.png`
        }
          , be = i(( () => ie.value.data.bonusTemplateList.find((e => e.bonusTemplateId == re.value))))
          , fe = async (e=!1) => {
            var a, l, n, t, s;
            ye.value = "",
            e ? await te.fetchPaymentChannels() : await Promise.all([C.fetchPaymentListInfo(), te.fetchPaymentChannels(), se.getUserProfile()]),
            ie && (pe.value = null == (l = null == (a = null == ie ? void 0 : ie.value) ? void 0 : a.depositPaymentList) ? void 0 : l[0].paymentTypeList),
            ie && 0 !== oe.value && (oe.value = null == (s = null == (t = null == (n = null == ie ? void 0 : ie.value) ? void 0 : n.depositPaymentList) ? void 0 : t[0]) ? void 0 : s.bankTypeId)
        }
        ;
        o(( () => {
            fe()
        }
        )),
        d(( () => me.value.success), (async e => {
            e && (await fe(!0),
            ( (e, a, l) => {
                var n, t;
                if (!e)
                    return;
                oe.value = e;
                const s = he.value.find((a => a.bankTypeId === e));
                s && (pe.value = s.paymentTypeList,
                pe.value.some((e => e.paymentTypeId === a)) ? (ue.value = a,
                ce.value = l) : (ue.value = null == (n = pe.value[0]) ? void 0 : n.paymentTypeId,
                ce.value = null == (t = pe.value[0]) ? void 0 : t.depositTypeId))
            }
            )(oe.value, ue.value, ce.value))
        }
        )),
        d([oe], ( () => {
            var e, a, l, n;
            ie && (ue.value = null == (a = null == (e = null == pe ? void 0 : pe.value) ? void 0 : e[0]) ? void 0 : a.paymentTypeId,
            ce.value = null == (n = null == (l = null == pe ? void 0 : pe.value) ? void 0 : l[0]) ? void 0 : n.depositTypeId)
        }
        )),
        d([ue], ( () => {
            C.handleDepositSetting()
        }
        ));
        const he = i(( () => {
            var e, a;
            const l = [];
            return null == (a = null == (e = ie.value) ? void 0 : e.depositPaymentList) || a.forEach((e => {
                var a;
                4194304 === e.bankTypeId ? null == (a = e.paymentTypeList) || a.forEach(( (a, n) => {
                    l.push({
                        bankTypeId: `${e.bankTypeId}-${n}`,
                        name: a.displayName,
                        isMaintain: e.isMaintain,
                        paymentMethodStatus: e.paymentMethodStatus,
                        startTime: e.startTime,
                        endTime: e.endTime,
                        paymentTypeList: [a]
                    })
                }
                )) : l.push({
                    ...e,
                    name: e.name
                })
            }
            )),
            l
        }
        ));
        return (e, a) => {
            var l, n, t, s, i, o, d, $, C;
            const M = b
              , D = T
              , te = I
              , se = P
              , me = U;
            return p(),
            r(y, null, [u("div", E, [m(ie).loading ? (p(),
            r("div", N, [u("div", S, [(p(),
            r(y, null, v(4, (e => c(M, {
                key: e,
                class: g("" + (m(ne).isWeb ? "h-24 w-24" : "h-24 w-36")),
                ui: {
                    background: "bg-gray-200 dark:bg-gray-700"
                }
            }, null, 8, ["class"]))), 64))]), u("div", null, [c(M, {
                class: g("" + (m(ne).isWeb ? "h-80 w-full" : "h-72 w-full")),
                ui: {
                    background: "bg-gray-200 dark:bg-gray-700"
                }
            }, null, 8, ["class"])])])) : (p(),
            r("div", K, [u("div", V, [u("label", q, x(e.$t("deposit.promotion")), 1), c(te, {
                name: "bonusTemplate",
                class: "w-full lg:w-[60%]"
            }, {
                default: k(( () => [c(D, {
                    modelValue: m(re),
                    "onUpdate:modelValue": a[0] || (a[0] = e => w(re) ? re.value = e : null),
                    color: "white",
                    variant: "none",
                    options: m(de),
                    size: "xl",
                    "selected-icon": "",
                    "option-attribute": "label",
                    "value-attribute": "value",
                    placeholder: e.$t("deposit.selectPromotion"),
                    ui: {
                        placeholder: "text-gray-800 dark:text-white font-bold",
                        variant: {
                            none: "bg-white dark:bg-zinc-850 h-13 font-bold text-sm  focus:ring-1 focus:ring-primary-500 placeholder:text-white"
                        }
                    },
                    "ui-menu": {
                        background: "dark:bg-zinc-850",
                        base: "font-bold",
                        ring: "ring-0",
                        option: {
                            active: "bg-gray-100 dark:bg-gray-600",
                            selected: "!text-primary"
                        }
                    }
                }, null, 8, ["modelValue", "options", "placeholder"])])),
                _: 1
            })]), u("section", A, [u("label", null, x(e.$t("deposit.paymentMethod")), 1), u("div", H, [(p(!0),
            r(y, null, v(m(he), ( (e, a) => {
                var l, n, t, s;
                return p(),
                r("div", {
                    key: a,
                    class: "relative"
                }, [u("div", {
                    class: g([`\n              ${m(oe) === (null == e ? void 0 : e.bankTypeId) ? "border-primary" : "border-transparent"} \n              `, "py-2 bg-white dark:bg-zinc-850 w-full h-[95px] flex flex-col items-center justify-center cursor-pointer rounded-md border-[1px]"]),
                    onClick: a => {
                        oe.value = null == e ? void 0 : e.bankTypeId,
                        pe.value = null == e ? void 0 : e.paymentTypeList
                    }
                }, [c(se, {
                    class: g(["object-contain overflow-hidden aspect-square", [String(null == e ? void 0 : e.bankTypeId).includes("4194304") ? "h-[4.75rem] pt-3" : "h-[70%]"]]),
                    src: ge(null == e ? void 0 : e.name)
                }, null, 8, ["class", "src"]), u("p", {
                    class: g(["text-center text-xs sm:text-sm whitespace-nowrap", [String(null == e ? void 0 : e.bankTypeId).includes("4194304") && "pt-[7px]"]])
                }, x(1 === e.bankTypeId ? "Local Bank" : null == e ? void 0 : e.name), 3), !(null == (n = null == (l = m(be)) ? void 0 : l.bonusAmount) ? void 0 : n.includes("X")) && (null == (s = null == (t = m(be)) ? void 0 : t.awardingBankTypes) ? void 0 : s.includes(null == e ? void 0 : e.bankTypeId)) ? (p(),
                h(j, {
                    key: 0,
                    name: "chip",
                    appear: "",
                    "enter-active-class": "transition transform duration-300 ease-in-out",
                    "enter-from-class": "opacity-0 scale-75",
                    "enter-to-class": "opacity-100 scale-100"
                }, {
                    default: k(( () => [u("div", F, [u("div", {
                        innerHTML: `+ ${m(be).bonusAmount}`,
                        class: "text-white font-bold text-xs px-[2px] py-[2px] tracking-tighter"
                    }, null, 8, G)])])),
                    _: 1
                })) : f("", !0)], 10, W)])
            }
            )), 128)), u("div", {
                class: g([(0 === m(oe) ? "border-primary" : "border-transparent") + " \n            ", "py-2 bg-white dark:bg-zinc-850 w-full h-[95px] flex flex-col items-center justify-center cursor-pointer rounded-md border-2 pt-4"]),
                onClick: a[1] || (a[1] = e => oe.value = 0)
            }, [c(se, {
                class: "object-contain h-[3rem] overflow-hidden aspect-square",
                src: "/images/jwPoints.png"
            }), u("p", Q, x(e.$t("jwPoints")), 1)], 2)]), ("number" == typeof m(oe) ? m(oe) : null == (l = m(oe)) ? void 0 : l.split("-")[0]) > 0 && !(null == (t = null == (n = m(ie).data) ? void 0 : n.verifyTips) ? void 0 : t.isRestrictionDeposit) ? (p(),
            r("div", X, [u("div", J, [(p(!0),
            r(y, null, v(m(pe), ( (e, a) => (p(),
            r("div", {
                key: a
            }, [u("div", {
                class: g([`${m(ue) === e.paymentTypeId ? "border-primary" : "border-transparent"} ${m(ne).isWeb ? "h-20 w-20" : ""}`, "px-4 py-3 flex flex-col gap-0 items-center justify-center bg-gray-50 dark:bg-zinc-850 rounded-md border-2 cursor-pointer"]),
                onClick: a => {
                    ue.value = null == e ? void 0 : e.paymentTypeId,
                    ce.value = null == e ? void 0 : e.depositTypeId
                }
            }, [u("p", Z, x((null == e ? void 0 : e.displayName) || ""), 1)], 10, Y)])))), 128))])])) : f("", !0)]), !(null == (i = null == (s = m(ie).data) ? void 0 : s.verifyTips) ? void 0 : i.isRestrictionDeposit) && ("number" == typeof m(oe) ? m(oe) : null == (o = m(oe)) ? void 0 : o.split("-")[0]) > 0 ? (p(),
            r("div", ee, [c(m(B))])) : f("", !0), 0 === m(oe) ? (p(),
            r("div", ae, [c(m(_))])) : ("number" == typeof m(oe) ? m(oe) : null == (d = m(oe)) ? void 0 : d.split("-")[0]) > 0 ? (p(),
            h(L((null == (C = null == ($ = m(ie).data) ? void 0 : $.verifyTips) ? void 0 : C.isRestrictionDeposit) ? m(R) : m(O)), {
                key: 2
            })) : (p(),
            r("div", le, [c(z, {
                channel: m(ye)
            }, null, 8, ["channel"])]))]))]), c(me)], 64)
        }
    }
};
export {ne as default};
