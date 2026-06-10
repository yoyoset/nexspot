import React from "react";
import { Heart } from "lucide-react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import alipayImg from "../../../assets/donate_alipay.png";
import wechatImg from "../../../assets/donate_wechat.png";
import paypalImg from "../../../assets/donate_paypal.png";
import logoImg from "../../../assets/sulogo.jpg";

const DonateTab: React.FC = () => {
    const { i18n } = useTranslation();
    const isZh = i18n.language === 'zh' || i18n.language === 'zh-CN';

    const title = isZh ? "赞助一点 Token" : "Sponsor a few tokens";
    const blessing = isZh
        ? "如果 NexSpot 提升了你的效率，欢迎赞助一点 Token。你的支持会直接变成下一轮的推理算力，让新功能持续产出。"
        : "If NexSpot improved your workflow, consider sponsoring a few tokens. Your support converts directly into inference compute that keeps new features shipping.";

    const qrs = [
        { img: alipayImg, label: isZh ? "支付宝" : "Alipay" },
        { img: wechatImg, label: isZh ? "微信支付" : "WeChat Pay" },
        { img: paypalImg, label: "PayPal" },
    ];

    return (
        <motion.div key="donate" initial={{ y: 7 }} animate={{ y: 0 }} transition={{ duration: 0.18 }} className="max-w-[620px] flex flex-col gap-6">
            {/* Hero card */}
            <div className="relative overflow-hidden rounded-lg border border-line bg-bg-1 p-6 flex flex-col items-center text-center gap-4">
                <div className="absolute inset-x-0 top-0 h-24 bg-accent-soft pointer-events-none" />
                <div className="relative">
                    <img src={logoImg} alt="Dev" className="w-20 h-20 rounded-full object-cover border-4 border-bg-1 shadow-sm" />
                    <div className="absolute -bottom-1 -right-1 w-8 h-8 rounded-full bg-accent flex items-center justify-center border-2 border-bg-1">
                        <Heart className="w-4 h-4 text-on-accent fill-current" />
                    </div>
                </div>
                <div className="relative flex flex-col gap-1.5">
                    <h2 className="text-[16px] font-extrabold text-ink">{title}</h2>
                    <p className="text-[12.5px] text-muted leading-relaxed max-w-[420px]">{blessing}</p>
                </div>
            </div>

            {/* QR codes */}
            <div className="grid grid-cols-3 gap-4">
                {qrs.map((q) => (
                    <div key={q.label} className="flex flex-col items-center gap-2.5">
                        <div className="p-2 rounded-panel bg-bg-1 border border-line">
                            <img src={q.img} alt={q.label} className="w-full aspect-square rounded object-contain" />
                        </div>
                        <span className="text-[12px] font-semibold text-muted">{q.label}</span>
                    </div>
                ))}
            </div>
        </motion.div>
    );
};

export default DonateTab;
